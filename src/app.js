const $ = (selector, root = document) => root.querySelector(selector);
const $$ = (selector, root = document) => [...root.querySelectorAll(selector)];
const invoke = window.__TAURI__?.core?.invoke;
const preview = $('#preview');
const ctx = preview.getContext('2d', { alpha: false });
const homePreview = $('#homePreview');
const homeCtx = homePreview.getContext('2d', { alpha: false });

const FALLBACK_PROFILE = {
  width: 1920,
  height: 1080,
  fps: 60,
  frame_count: 12267,
  duration_seconds: 204.45,
  geometry: {
    card_pitch_px: 477,
    artwork_bottom_y: 871,
    title_top_y: 872,
    title_bottom_y: 964,
    description_top_y: 965,
    description_bottom_y: 1074,
    bottom_border_top_y: 1075,
    separator_nominal_px: 6,
  },
};

const SAMPLE_CARDS = [
  ['10', 'SECONDS OLD', 'Breathing', "A baby's first breath requires blood flow through the heart."],
  ['1', 'HOUR OLD', 'Suckling', 'Newborns instinctively try to feed within just hours.'],
  ['3', 'DAYS OLD', "Recognizing Mom's Smell", 'Within days a baby can recognize a familiar scent.'],
  ['6.5', 'MONTHS OLD', 'Recognizing Their Own Name', 'A baby turns toward their name months before speaking.'],
  ['8', 'MONTHS OLD', 'Object Permanence', 'Objects still exist even when they are out of sight.'],
];

const state = {
  profile: FALLBACK_PROFILE,
  track: [],
  scene: null,
  frame: 0,
  playing: false,
  playStartFrame: 0,
  playStartWall: 0,
  raf: 0,
  selectedCardId: null,
  imageCache: new Map(),
  audioElements: new Map(),
  search: '',
  project: null,
};

let toastTimer;
let sceneRequest = 0;

function uid(prefix = 'item') {
  return globalThis.crypto?.randomUUID?.() ?? `${prefix}-${Date.now()}-${Math.random().toString(16).slice(2)}`;
}

function makeCard(index = 0, values = {}) {
  return {
    id: values.id || uid('card'),
    badge: values.badge ?? values.badge_primary ?? String(index + 1),
    badgeSubtitle: values.badgeSubtitle ?? values.badge_secondary ?? '',
    title: values.title ?? `Card ${index + 1}`,
    description: values.description ?? '',
    artwork: values.artwork ?? values.image ?? null,
    artworkName: values.artworkName ?? null,
    accent: values.accent || '#e00000',
    background: values.background || '#138ddb',
  };
}

function defaultProject() {
  return {
    version: 3,
    name: 'Untitled comparison',
    cards: SAMPLE_CARDS.map((row, index) => makeCard(index, {
      badge: row[0], badgeSubtitle: row[1], title: row[2], description: row[3],
    })),
    settings: {
      modelId: 'reference_locked',
      automaticTiming: true,
      customDuration: null,
      soundtrackMasterVolume: 1,
    },
    audioTracks: [],
  };
}

function normalizeProject(input) {
  if (!input || typeof input !== 'object') return defaultProject();
  const cards = Array.isArray(input.cards) ? input.cards.map((card, index) => makeCard(index, card || {})) : [];
  const oldEmptyShell = Number(input.version || 0) < 3 && cards.length === 0;
  if (oldEmptyShell) return defaultProject();
  const settings = input.settings || {};
  const audioTracks = Array.isArray(input.audioTracks)
    ? input.audioTracks
    : Array.isArray(input.audio_tracks) ? input.audio_tracks : [];
  return {
    version: 3,
    name: String(input.name || 'Untitled comparison'),
    cards,
    settings: {
      modelId: 'reference_locked',
      automaticTiming: settings.automaticTiming ?? settings.automatic_timing ?? true,
      customDuration: Number.isFinite(Number(settings.customDuration ?? settings.custom_duration))
        ? Number(settings.customDuration ?? settings.custom_duration) : null,
      soundtrackMasterVolume: Number.isFinite(Number(settings.soundtrackMasterVolume ?? settings.soundtrack_master_volume))
        ? Math.max(0, Number(settings.soundtrackMasterVolume ?? settings.soundtrack_master_volume)) : 1,
    },
    audioTracks: audioTracks.map((track) => ({
      id: track.id || uid('track'),
      name: track.name || 'Soundtrack',
      data: track.data || track.path || null,
      duration: Number(track.duration || 0),
      startTime: Number(track.startTime ?? track.start_time ?? 0),
      trimStart: Number(track.trimStart ?? track.trim_start ?? 0),
      trimEnd: track.trimEnd ?? track.trim_end ?? null,
      volume: Number(track.volume ?? 1),
      fadeIn: Number(track.fadeIn ?? track.fade_in ?? 0),
      fadeOut: Number(track.fadeOut ?? track.fade_out ?? 0),
      loop: Boolean(track.loop),
    })),
  };
}

function restoreProject() {
  try {
    const raw = localStorage.getItem('watchcompare.project.v3') || localStorage.getItem('watchcompare.project.v1');
    state.project = raw ? normalizeProject(JSON.parse(raw)) : defaultProject();
  } catch {
    state.project = defaultProject();
  }
  state.selectedCardId = state.project.cards[0]?.id || null;
}

function persistProject() {
  try {
    const raw = JSON.stringify(state.project);
    if (raw.length < 4_000_000) localStorage.setItem('watchcompare.project.v3', raw);
  } catch (error) {
    console.warn('Could not persist project', error);
  }
}

function showToast(message) {
  const toast = $('#toast');
  toast.textContent = message;
  toast.classList.add('show');
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => toast.classList.remove('show'), 2600);
}

function safeFilename(value) {
  return String(value || 'watchcompare').trim().replace(/[^a-z0-9._-]+/gi, '-').replace(/^-+|-+$/g, '') || 'watchcompare';
}

function downloadBlob(blob, filename) {
  const file = new File([blob], filename, { type: blob.type || 'application/octet-stream' });
  if (navigator.canShare?.({ files: [file] })) {
    navigator.share({ files: [file], title: filename }).catch((error) => {
      if (error?.name !== 'AbortError') fallbackDownload(blob, filename);
    });
  } else {
    fallbackDownload(blob, filename);
  }
}

function fallbackDownload(blob, filename) {
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  a.remove();
  setTimeout(() => URL.revokeObjectURL(url), 2000);
}

function setView(view) {
  $$('.view').forEach((node) => node.classList.toggle('active', node.id === `${view}View`));
  $$('[data-view]').forEach((node) => node.classList.toggle('active', node.dataset.view === view));
  const title = { home: 'WatchCompare', data: 'Comparison data', preview: 'Preview', audio: 'Soundtrack', export: 'Export' }[view] || 'WatchCompare';
  $('#mobileTitle').textContent = title;
  if (view === 'preview') renderPreview();
  if (view === 'audio') renderAudioTracks();
  window.scrollTo({ top: 0, behavior: 'smooth' });
}

function parseDelimited(text) {
  const sample = text.slice(0, 4096);
  const delimiter = sample.includes('\t') ? '\t' : (sample.split(';').length > sample.split(',').length ? ';' : ',');
  const rows = [];
  let row = [], field = '', quoted = false;
  for (let i = 0; i < text.length; i += 1) {
    const ch = text[i];
    if (quoted) {
      if (ch === '"' && text[i + 1] === '"') { field += '"'; i += 1; }
      else if (ch === '"') quoted = false;
      else field += ch;
    } else if (ch === '"') quoted = true;
    else if (ch === delimiter) { row.push(field); field = ''; }
    else if (ch === '\n') { row.push(field.replace(/\r$/, '')); rows.push(row); row = []; field = ''; }
    else field += ch;
  }
  row.push(field.replace(/\r$/, ''));
  if (row.some((value) => value.length)) rows.push(row);
  return rows.filter((r) => r.some((value) => String(value).trim()));
}

function normalizedHeader(value) {
  return String(value).trim().toLowerCase().replace(/[\s_\-/]+/g, '');
}

function cardsFromRows(rows) {
  if (rows.length < 2) throw new Error('Add a header row and at least one card row.');
  const headers = rows[0].map(normalizedHeader);
  const find = (...aliases) => headers.findIndex((header) => aliases.includes(header));
  const badge = find('badge', 'badgevalue', 'value', 'date', 'year', 'rank', 'amount', 'age');
  const badgeLabel = find('badgelabel', 'badgesubtitle', 'unit', 'units', 'label', 'metric');
  const title = find('title', 'name', 'heading', 'item', 'subject');
  const description = find('description', 'desc', 'details', 'summary', 'text', 'caption');
  const artwork = find('artwork', 'image', 'imagepath', 'imageurl', 'photo', 'picture', 'thumbnail');
  return rows.slice(1).map((row, index) => makeCard(index, {
    badge: badge >= 0 ? String(row[badge] || '').trim() : '',
    badgeSubtitle: badgeLabel >= 0 ? String(row[badgeLabel] || '').trim() : '',
    title: title >= 0 ? String(row[title] || '').trim() : '',
    description: description >= 0 ? String(row[description] || '').trim() : '',
    artwork: artwork >= 0 ? String(row[artwork] || '').trim() || null : null,
    artworkName: artwork >= 0 && row[artwork] ? String(row[artwork]).split('/').pop() : null,
  }));
}

function refreshDataDetection() {
  const text = $('#dataPaste').value;
  if (!text.trim()) {
    $('#dataDetection').textContent = 'Paste CSV/TSV data here.';
    $('#applyDataButton').disabled = true;
    return;
  }
  try {
    const rows = parseDelimited(text);
    const cards = cardsFromRows(rows);
    $('#dataDetection').textContent = `Ready · ${cards.length} ${cards.length === 1 ? 'card' : 'cards'} · ${rows[0].length} fields`;
    $('#applyDataButton').disabled = cards.length === 0;
  } catch (error) {
    $('#dataDetection').textContent = error.message;
    $('#applyDataButton').disabled = true;
  }
}

function applyPastedData() {
  try {
    const cards = cardsFromRows(parseDelimited($('#dataPaste').value));
    state.project.cards = cards;
    state.selectedCardId = cards[0]?.id || null;
    persistProject();
    renderCards();
    renderPreview();
    showToast(`Created ${cards.length} cards.`);
  } catch (error) {
    showToast(error.message);
  }
}

function fileToDataUrl(file) {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(reader.result);
    reader.onerror = reject;
    reader.readAsDataURL(file);
  });
}

async function importDataFile(file) {
  const lower = file.name.toLowerCase();
  if (lower.endsWith('.xlsx')) {
    showToast('XLSX import is being restored in the Rust backend; use CSV/TSV in this build.');
    return;
  }
  const text = await file.text();
  $('#dataPaste').value = text;
  refreshDataDetection();
  applyPastedData();
}

function currentCard() {
  return state.project.cards.find((card) => card.id === state.selectedCardId) || null;
}

function renderCards() {
  const grid = $('#cardGrid');
  grid.textContent = '';
  const query = (state.search || '').trim().toLowerCase();
  const cards = state.project.cards.filter((card) => !query || [card.title, card.description, card.badge, card.badgeSubtitle].some((v) => String(v).toLowerCase().includes(query)));
  $('#cardCount').textContent = `${state.project.cards.length} ${state.project.cards.length === 1 ? 'card' : 'cards'}`;
  cards.forEach((card) => {
    const pin = document.createElement('article');
    pin.className = 'card-pin';
    const visual = document.createElement('div');
    visual.className = 'card-pin-visual';
    visual.style.background = card.background;
    if (card.artwork) {
      const img = document.createElement('img');
      img.src = card.artwork;
      visual.appendChild(img);
    }
    const badge = document.createElement('div');
    badge.className = 'card-pin-badge';
    badge.style.background = card.accent;
    badge.textContent = card.badge || '—';
    const small = document.createElement('small');
    small.textContent = card.badgeSubtitle || '';
    badge.appendChild(small);
    visual.appendChild(badge);
    const copy = document.createElement('div');
    copy.className = 'card-pin-copy';
    copy.innerHTML = `<strong></strong><span></span>`;
    $('strong', copy).textContent = card.title || 'Untitled card';
    $('span', copy).textContent = card.description || 'No description';
    pin.append(visual, copy);
    pin.addEventListener('click', () => openCardEditor(card.id));
    grid.appendChild(pin);
  });
}

function openCardEditor(id) {
  state.selectedCardId = id;
  const card = currentCard();
  if (!card) return;
  const index = state.project.cards.indexOf(card);
  $('#cardEditorHeading').textContent = `Card ${index + 1}`;
  $('#cardBadge').value = card.badge;
  $('#cardBadgeSubtitle').value = card.badgeSubtitle;
  $('#cardTitle').value = card.title;
  $('#cardDescription').value = card.description;
  $('#cardAccent').value = card.accent;
  $('#cardBackground').value = card.background;
  $('#artworkName').textContent = card.artworkName || 'No image';
  $('#moveLeftButton').disabled = index === 0;
  $('#moveRightButton').disabled = index === state.project.cards.length - 1;
  $('#cardEditor').hidden = false;
}

function closeCardEditor() {
  $('#cardEditor').hidden = true;
}

function commitCardEditor() {
  const card = currentCard();
  if (!card) return;
  card.badge = $('#cardBadge').value;
  card.badgeSubtitle = $('#cardBadgeSubtitle').value;
  card.title = $('#cardTitle').value;
  card.description = $('#cardDescription').value;
  card.accent = $('#cardAccent').value;
  card.background = $('#cardBackground').value;
  persistProject();
  renderCards();
  renderPreview();
}

function addCard() {
  const card = makeCard(state.project.cards.length);
  state.project.cards.push(card);
  state.selectedCardId = card.id;
  persistProject();
  renderCards();
  openCardEditor(card.id);
}

function moveCard(delta) {
  const card = currentCard();
  const index = state.project.cards.indexOf(card);
  const target = index + delta;
  if (index < 0 || target < 0 || target >= state.project.cards.length) return;
  state.project.cards.splice(index, 1);
  state.project.cards.splice(target, 0, card);
  persistProject();
  renderCards();
  openCardEditor(card.id);
  renderPreview();
}

function duplicateCard() {
  const card = currentCard();
  if (!card) return;
  const index = state.project.cards.indexOf(card);
  const copy = { ...card, id: uid('card'), title: `${card.title || 'Card'} copy` };
  state.project.cards.splice(index + 1, 0, copy);
  state.selectedCardId = copy.id;
  persistProject();
  renderCards();
  openCardEditor(copy.id);
  renderPreview();
}

function deleteCard() {
  const card = currentCard();
  if (!card) return;
  const index = state.project.cards.indexOf(card);
  state.project.cards.splice(index, 1);
  state.selectedCardId = state.project.cards[Math.min(index, state.project.cards.length - 1)]?.id || null;
  persistProject();
  closeCardEditor();
  renderCards();
  renderPreview();
}

function getFrameState(frame) {
  return state.track[frame] || {
    frame,
    time_millis: frame / 60 * 1000,
    time_seconds: frame / 60,
    stage: frame < 630 ? 'intro' : frame < 11843 ? 'cruise' : frame < 12180 ? 'outro' : 'fade',
    card_train_x_px: frame < 630 ? 0 : -313.5 - (frame - 630) * 2.224552,
    card_phase_px: 0,
  };
}

function easeOutCubic(t) {
  const x = Math.max(0, Math.min(1, t));
  return 1 - Math.pow(1 - x, 3);
}

function ensureImage(source) {
  if (!source) return null;
  if (state.imageCache.has(source)) {
    const cached = state.imageCache.get(source);
    return cached.ready ? cached.image : null;
  }
  const image = new Image();
  const record = { image, ready: false };
  state.imageCache.set(source, record);
  image.onload = () => { record.ready = true; renderPreview(); };
  image.onerror = () => state.imageCache.delete(source);
  image.src = source;
  return null;
}

function drawCover(image, x, y, width, height) {
  const sourceRatio = image.width / image.height;
  const targetRatio = width / height;
  let sx = 0, sy = 0, sw = image.width, sh = image.height;
  if (sourceRatio > targetRatio) { sw = image.height * targetRatio; sx = (image.width - sw) / 2; }
  else { sh = image.width / targetRatio; sy = (image.height - sh) / 2; }
  ctx.drawImage(image, sx, sy, sw, sh, x, y, width, height);
}

function drawBadge(card, x, width) {
  const bw = 246, bh = 282, bx = x + (width - bw) / 2, by = 68;
  ctx.save();
  ctx.beginPath();
  ctx.moveTo(bx + bw * .5, by);
  ctx.lineTo(bx + bw, by + bh * .245);
  ctx.lineTo(bx + bw, by + bh * .745);
  ctx.lineTo(bx + bw * .5, by + bh);
  ctx.lineTo(bx, by + bh * .745);
  ctx.lineTo(bx, by + bh * .245);
  ctx.closePath();
  const gradient = ctx.createLinearGradient(0, by, 0, by + bh);
  gradient.addColorStop(0, '#eb0909');
  gradient.addColorStop(.5, card.accent || '#e00000');
  gradient.addColorStop(1, '#d50000');
  ctx.fillStyle = gradient;
  ctx.fill();
  ctx.strokeStyle = '#ff4545';
  ctx.lineWidth = 2;
  ctx.stroke();
  ctx.fillStyle = '#fff';
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  ctx.font = '900 58px Arial, sans-serif';
  ctx.fillText(card.badge || '', bx + bw / 2, by + 117);
  ctx.font = '900 21px Arial, sans-serif';
  ctx.fillText(String(card.badgeSubtitle || '').toUpperCase(), bx + bw / 2, by + 166);
  ctx.restore();
}

function drawCard(card, x, index, frameState) {
  const pitch = state.profile.geometry.card_pitch_px || 477;
  const separator = 6;
  const innerWidth = pitch - separator;
  const introStart = 5 + index * 120;
  if (frameState.stage === 'intro' && frameState.frame < introStart) return;
  let reveal = 1;
  if (frameState.stage === 'intro') reveal = easeOutCubic((frameState.frame - introStart) / 76);
  const visibleWidth = Math.max(0, innerWidth * reveal);
  if (visibleWidth <= 0) return;

  ctx.save();
  ctx.beginPath();
  ctx.rect(x, 0, visibleWidth, 1080);
  ctx.clip();
  const artH = 872;
  ctx.fillStyle = card.background || '#138ddb';
  ctx.fillRect(x, 0, innerWidth, artH);
  const image = ensureImage(card.artwork);
  if (image) drawCover(image, x, 0, innerWidth, artH);
  else {
    const g = ctx.createLinearGradient(0, 0, 0, artH);
    g.addColorStop(0, card.background || '#138ddb');
    g.addColorStop(1, '#0b74be');
    ctx.fillStyle = g;
    ctx.fillRect(x, 0, innerWidth, artH);
  }
  drawBadge(card, x, innerWidth);

  ctx.fillStyle = '#f0f0f0';
  ctx.fillRect(x, 872, innerWidth, 93);
  ctx.fillStyle = '#101010';
  ctx.font = '900 40px Arial, sans-serif';
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  ctx.fillText(card.title || '', x + innerWidth / 2, 918, innerWidth - 34);

  ctx.fillStyle = '#625f56';
  ctx.fillRect(x, 965, innerWidth, 110);
  ctx.fillStyle = '#fff';
  ctx.font = '700 25px Arial, sans-serif';
  const text = card.description || '';
  const words = text.split(/\s+/);
  const lines = [];
  let line = '';
  for (const word of words) {
    const test = line ? `${line} ${word}` : word;
    if (ctx.measureText(test).width > innerWidth - 34 && line) { lines.push(line); line = word; }
    else line = test;
  }
  if (line) lines.push(line);
  lines.slice(0, 3).forEach((entry, i) => ctx.fillText(entry, x + innerWidth / 2, 994 + i * 28, innerWidth - 34));
  ctx.fillStyle = '#11100c';
  ctx.fillRect(x, 1075, pitch, 5);
  ctx.restore();

  ctx.fillStyle = '#11100c';
  ctx.fillRect(x + innerWidth, 0, separator, 1080);
}

function renderPreview() {
  const frameState = getFrameState(state.frame);
  ctx.fillStyle = '#101010';
  ctx.fillRect(0, 0, 1920, 1080);
  const pitch = state.profile.geometry.card_pitch_px || 477;
  if (!state.project.cards.length) {
    ctx.fillStyle = '#eee';
    ctx.font = '800 58px Arial, sans-serif';
    ctx.textAlign = 'center';
    ctx.fillText('Add data to start', 960, 515);
    ctx.fillStyle = '#888';
    ctx.font = '28px Arial, sans-serif';
    ctx.fillText('CTS workflow · WatchCompare renderer', 960, 565);
  } else {
    state.project.cards.forEach((card, index) => {
      const x = Math.round(index * pitch + frameState.card_train_x_px);
      if (x < 1920 && x + pitch > 0) drawCard(card, x, index, frameState);
    });
  }
  if (state.scene?.outro_wipe_bottom_y != null) {
    const bottom = Number(state.scene.outro_wipe_bottom_y);
    ctx.fillStyle = '#000';
    ctx.fillRect(0, 0, 1920, Math.max(0, Math.min(1080, bottom + 1)));
  }
  const fade = Number(state.scene?.outro_fade_level ?? 1);
  if (fade < 1) {
    ctx.fillStyle = `rgba(0,0,0,${1 - fade})`;
    ctx.fillRect(0, 0, 1920, 1080);
  }
  homeCtx.clearRect(0, 0, homePreview.width, homePreview.height);
  homeCtx.drawImage(preview, 0, 0, homePreview.width, homePreview.height);
  $('#stageValue').textContent = frameState.stage;
  $('#pitchValue').textContent = `${pitch} px`;
  $('#homePreviewSummary').textContent = `${state.project.cards.length} cards · ${formatDuration(projectDuration())} · frame ${state.frame.toLocaleString()}`;
}

function requestScene(frame) {
  if (!invoke || !(frame < 430 || frame >= 11868)) { state.scene = null; return; }
  const id = ++sceneRequest;
  invoke('reference_scene', { frame }).then((scene) => {
    if (id === sceneRequest && state.frame === frame) { state.scene = scene; renderPreview(); }
  }).catch(() => {});
}

function computeAutoDuration() {
  const count = state.project.cards.length;
  if (!count) return 0;
  return Math.min(count, 4) * 2 + Math.max(0, count - 4) * (10 / 3) + 2 + .8;
}

function projectDuration() {
  if (!state.project.settings.automaticTiming && Number(state.project.settings.customDuration) > 0) return Number(state.project.settings.customDuration);
  return computeAutoDuration() || state.profile.duration_seconds;
}

function outputTimeForFrame(frame) {
  return (frame / Math.max(1, state.profile.frame_count - 1)) * projectDuration();
}

function setFrame(frame, { syncAudio = true } = {}) {
  state.frame = Math.max(0, Math.min(state.profile.frame_count - 1, Math.round(frame)));
  $('#timeline').value = String(state.frame);
  const seconds = outputTimeForFrame(state.frame);
  $('#timeReadout').textContent = formatDuration(seconds, true);
  $('#frameReadout').textContent = `Frame ${state.frame.toLocaleString()}`;
  requestScene(state.frame);
  renderPreview();
  if (syncAudio) syncAudio(false);
}

function formatDuration(seconds, millis = false) {
  const total = Math.max(0, Number(seconds) || 0);
  const m = Math.floor(total / 60);
  const s = Math.floor(total % 60);
  if (millis) return `${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}.${String(Math.floor((total % 1) * 1000)).padStart(3, '0')}`;
  return `${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`;
}

function parseDuration(value) {
  const parts = String(value).trim().split(':').map(Number);
  if (!parts.length || parts.some((v) => !Number.isFinite(v) || v < 0)) return null;
  if (parts.length === 1) return parts[0];
  if (parts.length === 2) return parts[0] * 60 + parts[1];
  if (parts.length === 3) return parts[0] * 3600 + parts[1] * 60 + parts[2];
  return null;
}

function updateDurationUi() {
  const automatic = state.project.settings.automaticTiming;
  $('#automaticTiming').checked = automatic;
  $('#customDuration').disabled = automatic;
  $('#customDuration').value = automatic || !state.project.settings.customDuration ? '' : formatDuration(state.project.settings.customDuration);
  $('#durationSummary').textContent = automatic
    ? `Automatic · ${formatDuration(computeAutoDuration())}`
    : `Custom · ${formatDuration(projectDuration())}`;
}

function play() {
  if (state.playing) return pause();
  if (state.frame >= state.profile.frame_count - 1) setFrame(0);
  state.playing = true;
  state.playStartFrame = state.frame;
  state.playStartWall = performance.now();
  $('#playButton').textContent = 'Ⅱ';
  syncAudio(true);
  const tick = (now) => {
    if (!state.playing) return;
    const durationMs = projectDuration() * 1000;
    const modelFramesPerMs = (state.profile.frame_count - 1) / Math.max(1, durationMs);
    const next = state.playStartFrame + (now - state.playStartWall) * modelFramesPerMs;
    if (next >= state.profile.frame_count - 1) { setFrame(state.profile.frame_count - 1, { syncAudio: false }); pause(); return; }
    setFrame(next, { syncAudio: false });
    updateAudioVolumes(outputTimeForFrame(state.frame));
    state.raf = requestAnimationFrame(tick);
  };
  state.raf = requestAnimationFrame(tick);
}

function pause() {
  state.playing = false;
  cancelAnimationFrame(state.raf);
  $('#playButton').textContent = '▶';
  state.audioElements.forEach((audio) => audio.pause());
}

function audioElement(track) {
  if (!track.data) return null;
  let audio = state.audioElements.get(track.id);
  if (!audio) {
    audio = new Audio(track.data);
    audio.preload = 'auto';
    state.audioElements.set(track.id, audio);
  }
  return audio;
}

function trackSegmentDuration(track) {
  const end = Number(track.trimEnd) > Number(track.trimStart) ? Number(track.trimEnd) : Number(track.duration || 0);
  return Math.max(.001, end - Number(track.trimStart || 0));
}

function trackGain(track, timelineTime) {
  const local = timelineTime - Number(track.startTime || 0);
  if (local < 0) return 0;
  const segment = trackSegmentDuration(track);
  if (!track.loop && local >= segment) return 0;
  let gain = Math.max(0, Number(track.volume ?? 1)) * Math.max(0, Number(state.project.settings.soundtrackMasterVolume ?? 1));
  const fadeIn = Math.max(0, Number(track.fadeIn || 0));
  const fadeOut = Math.max(0, Number(track.fadeOut || 0));
  if (fadeIn && local < fadeIn) gain *= local / fadeIn;
  if (!track.loop && fadeOut && segment - local < fadeOut) gain *= Math.max(0, (segment - local) / fadeOut);
  return Math.min(1, gain);
}

function updateAudioVolumes(timelineTime) {
  state.project.audioTracks.forEach((track) => {
    const audio = audioElement(track);
    if (audio) audio.volume = trackGain(track, timelineTime);
  });
}

function syncAudio(shouldPlay = state.playing) {
  const timelineTime = outputTimeForFrame(state.frame);
  state.project.audioTracks.forEach((track) => {
    const audio = audioElement(track);
    if (!audio) return;
    const local = timelineTime - Number(track.startTime || 0);
    const segment = trackSegmentDuration(track);
    if (local < 0 || (!track.loop && local >= segment)) { audio.pause(); return; }
    const inSegment = track.loop ? (local % segment) : local;
    const target = Number(track.trimStart || 0) + inSegment;
    if (Math.abs((audio.currentTime || 0) - target) > .25) {
      try { audio.currentTime = target; } catch {}
    }
    audio.volume = trackGain(track, timelineTime);
    if (shouldPlay) audio.play().catch(() => {}); else audio.pause();
  });
}

async function addAudioFiles(files) {
  for (const file of files) {
    const data = await fileToDataUrl(file);
    const duration = await new Promise((resolve) => {
      const audio = new Audio(data);
      audio.addEventListener('loadedmetadata', () => resolve(Number(audio.duration || 0)), { once: true });
      audio.addEventListener('error', () => resolve(0), { once: true });
    });
    state.project.audioTracks.push({
      id: uid('track'), name: file.name, data, duration, startTime: 0, trimStart: 0,
      trimEnd: duration || null, volume: 1, fadeIn: 0, fadeOut: 0, loop: false,
    });
  }
  persistProject();
  renderAudioTracks();
}

function renderAudioTracks() {
  const list = $('#audioTrackList');
  list.textContent = '';
  $('#audioEmpty').hidden = state.project.audioTracks.length > 0;
  $('#masterVolume').value = String(state.project.settings.soundtrackMasterVolume ?? 1);
  $('#masterVolumeValue').textContent = `${Math.round((state.project.settings.soundtrackMasterVolume ?? 1) * 100)}%`;
  state.project.audioTracks.forEach((track) => {
    const card = document.createElement('article');
    card.className = 'audio-track-card surface';
    card.innerHTML = `
      <div class="track-name"><strong></strong><span></span></div>
      <label>Start<input data-key="startTime" type="number" min="0" step="0.1"></label>
      <label>Trim in<input data-key="trimStart" type="number" min="0" step="0.1"></label>
      <label>Trim out<input data-key="trimEnd" type="number" min="0" step="0.1"></label>
      <div class="track-wide">
        <label>Volume <input data-key="volume" type="range" min="0" max="1.5" step="0.01"></label>
        <label>Fade in <input data-key="fadeIn" type="number" min="0" step="0.1"></label>
        <label>Fade out <input data-key="fadeOut" type="number" min="0" step="0.1"></label>
        <label><input data-key="loop" type="checkbox"> Loop</label>
        <button class="tonal remove-track">Remove</button>
      </div>`;
    $('.track-name strong', card).textContent = track.name;
    $('.track-name span', card).textContent = `${track.duration ? formatDuration(track.duration, true) : 'duration unknown'} · starts ${track.startTime.toFixed(1)}s`;
    $$('[data-key]', card).forEach((input) => {
      const key = input.dataset.key;
      if (input.type === 'checkbox') input.checked = Boolean(track[key]);
      else input.value = track[key] ?? '';
      input.addEventListener('input', () => {
        track[key] = input.type === 'checkbox' ? input.checked : Number(input.value || 0);
        persistProject();
        syncAudio(false);
      });
    });
    $('.remove-track', card).addEventListener('click', () => {
      state.audioElements.get(track.id)?.pause();
      state.audioElements.delete(track.id);
      state.project.audioTracks = state.project.audioTracks.filter((item) => item.id !== track.id);
      persistProject();
      renderAudioTracks();
    });
    list.appendChild(card);
  });
}

async function importMegapack(file) {
  if (!invoke) throw new Error('Megapack import requires the app runtime.');
  showToast(`Opening ${file.name}…`);
  const bytes = Array.from(new Uint8Array(await file.arrayBuffer()));
  const project = await invoke('import_megapack', { bytes });
  pause();
  state.project = normalizeProject(project);
  state.selectedCardId = state.project.cards[0]?.id || null;
  persistProject();
  syncAllUi();
  setView('preview');
  showToast(`Megapack imported · ${state.project.cards.length} cards.`);
}

async function exportMegapack() {
  if (!invoke) throw new Error('Megapack export requires the app runtime.');
  showToast('Packing artwork and soundtrack…');
  const result = await invoke('export_megapack', { project: state.project });
  const bytes = result instanceof Uint8Array ? result : new Uint8Array(result);
  downloadBlob(new Blob([bytes], { type: 'application/zip' }), `${safeFilename(state.project.name)}.megapack.zip`);
}

function exportProject() {
  downloadBlob(new Blob([JSON.stringify(state.project, null, 2)], { type: 'application/json' }), `${safeFilename(state.project.name)}.watchcompare.json`);
}

function exportFrame() {
  preview.toBlob((blob) => {
    if (blob) downloadBlob(blob, `${safeFilename(state.project.name)}-frame-${String(state.frame).padStart(5, '0')}.png`);
  }, 'image/png');
}

function syncProjectName(value) {
  state.project.name = String(value || 'Untitled comparison');
  $('#desktopProjectName').value = state.project.name;
  persistProject();
}

function syncAllUi() {
  $('#desktopProjectName').value = state.project.name;
  renderCards();
  renderAudioTracks();
  updateDurationUi();
  renderPreview();
}

async function loadRenderer() {
  if (!invoke) {
    $('#rendererStatus').textContent = 'Browser fallback · exact Rust bridge unavailable';
    renderPreview();
    return;
  }
  try {
    const [profile, track] = await Promise.all([invoke('reference_profile'), invoke('reference_track')]);
    state.profile = profile;
    state.track = track;
    $('#timeline').max = String(profile.frame_count - 1);
    $('#rendererStatus').textContent = `Rust renderer connected · ${profile.frame_count.toLocaleString()} exact source frames`;
    $('#mobileSubtitle').textContent = 'CTS workflow · Rust renderer connected';
    renderPreview();
  } catch (error) {
    console.warn(error);
    $('#rendererStatus').textContent = 'Renderer bridge failed · using fallback geometry';
  }
}

function wireEvents() {
  $$('[data-view]').forEach((button) => button.addEventListener('click', () => setView(button.dataset.view)));
  $('#homeMegapackPin').addEventListener('click', () => $('#megapackFileInput').click());
  $('#desktopMegapackButton').addEventListener('click', () => $('#megapackFileInput').click());
  $('#importMegapackButton').addEventListener('click', () => $('#megapackFileInput').click());
  $('#homeImportDataPin').addEventListener('click', () => { setView('data'); $('#dataPaste').focus(); });
  $('#desktopProjectName').addEventListener('input', (event) => syncProjectName(event.target.value));
  $('#desktopSearch').addEventListener('input', (event) => { state.search = event.target.value; $('#cardSearch').value = state.search; renderCards(); });
  $('#cardSearch').addEventListener('input', (event) => { state.search = event.target.value; $('#desktopSearch').value = state.search; renderCards(); });
  $('#dataPaste').addEventListener('input', refreshDataDetection);
  $('#applyDataButton').addEventListener('click', applyPastedData);
  $('#importDataFileButton').addEventListener('click', () => $('#dataFileInput').click());
  $('#dataFileInput').addEventListener('change', async (event) => { const file = event.target.files?.[0]; if (file) await importDataFile(file); event.target.value = ''; });
  $('#addCardButton').addEventListener('click', addCard);
  $$('[data-close-editor]').forEach((node) => node.addEventListener('click', closeCardEditor));
  ['#cardBadge','#cardBadgeSubtitle','#cardTitle','#cardDescription','#cardAccent','#cardBackground'].forEach((selector) => $(selector).addEventListener('input', commitCardEditor));
  $('#moveLeftButton').addEventListener('click', () => moveCard(-1));
  $('#moveRightButton').addEventListener('click', () => moveCard(1));
  $('#duplicateCardButton').addEventListener('click', duplicateCard);
  $('#deleteCardButton').addEventListener('click', deleteCard);
  $('#chooseArtworkButton').addEventListener('click', () => $('#artworkFileInput').click());
  $('#artworkFileInput').addEventListener('change', async (event) => {
    const file = event.target.files?.[0]; const card = currentCard();
    if (file && card) { card.artwork = await fileToDataUrl(file); card.artworkName = file.name; $('#artworkName').textContent = file.name; persistProject(); renderCards(); renderPreview(); }
    event.target.value = '';
  });
  $('#timeline').addEventListener('input', (event) => { pause(); setFrame(Number(event.target.value)); });
  $('#playButton').addEventListener('click', play);
  $('#prevFrameButton').addEventListener('click', () => { pause(); setFrame(state.frame - 1); });
  $('#nextFrameButton').addEventListener('click', () => { pause(); setFrame(state.frame + 1); });
  $('#fitPreviewButton').addEventListener('click', () => $('#preview').scrollIntoView({ block: 'center', behavior: 'smooth' }));
  $('#exportFrameButton').addEventListener('click', exportFrame);
  $('#exportFrameButton2').addEventListener('click', exportFrame);
  $('#automaticTiming').addEventListener('change', (event) => { state.project.settings.automaticTiming = event.target.checked; persistProject(); updateDurationUi(); setFrame(state.frame); });
  $('#customDuration').addEventListener('change', (event) => {
    const seconds = parseDuration(event.target.value);
    if (!seconds) return showToast('Use MM:SS or HH:MM:SS.');
    state.project.settings.customDuration = seconds; persistProject(); updateDurationUi(); setFrame(state.frame);
  });
  $('#addAudioButton').addEventListener('click', () => $('#audioFileInput').click());
  $('#audioEmptyButton').addEventListener('click', () => $('#audioFileInput').click());
  $('#audioFileInput').addEventListener('change', async (event) => { if (event.target.files?.length) await addAudioFiles([...event.target.files]); event.target.value = ''; });
  $('#masterVolume').addEventListener('input', (event) => {
    state.project.settings.soundtrackMasterVolume = Number(event.target.value);
    $('#masterVolumeValue').textContent = `${Math.round(Number(event.target.value) * 100)}%`;
    updateAudioVolumes(outputTimeForFrame(state.frame)); persistProject();
  });
  $('#megapackFileInput').addEventListener('change', async (event) => {
    const file = event.target.files?.[0];
    if (file) { try { await importMegapack(file); } catch (error) { console.error(error); showToast(String(error)); } }
    event.target.value = '';
  });
  $('#exportMegapackButton').addEventListener('click', async () => { try { await exportMegapack(); } catch (error) { console.error(error); showToast(String(error)); } });
  $('#exportProjectButton').addEventListener('click', exportProject);
  $('#projectFileInput').addEventListener('change', async (event) => {
    const file = event.target.files?.[0];
    if (file) { try { state.project = normalizeProject(JSON.parse(await file.text())); state.selectedCardId = state.project.cards[0]?.id || null; persistProject(); syncAllUi(); showToast('Project imported.'); } catch { showToast('Could not read project JSON.'); } }
    event.target.value = '';
  });
  document.addEventListener('keydown', (event) => {
    if (event.key === 'Escape') closeCardEditor();
    if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'o') { event.preventDefault(); $('#projectFileInput').click(); }
    if (event.code === 'Space' && !['INPUT','TEXTAREA'].includes(document.activeElement?.tagName)) { event.preventDefault(); play(); }
  });
}

restoreProject();
wireEvents();
syncAllUi();
refreshDataDetection();
setFrame(0);
loadRenderer();
