const $ = (selector) => document.querySelector(selector);
const invoke = window.__TAURI__?.core?.invoke;
const canvas = $('#preview');
const ctx = canvas.getContext('2d', { alpha: false });
const canvasWrap = $('#canvasWrap');
const stageScroller = $('#stageScroller');
const emptyCanvasState = $('#emptyCanvasState');

const fallbackProfile = {
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

const state = {
  profile: fallbackProfile,
  track: [],
  scene: null,
  frame: 0,
  selectedId: null,
  playing: false,
  lastPlaybackTime: 0,
  frameAccumulator: 0,
  zoom: 'fit',
  sceneRequestId: 0,
  project: {
    version: 1,
    name: 'Untitled comparison',
    cards: [],
  },
};

const imageCache = new Map();
let toastTimer = null;

function uid() {
  return globalThis.crypto?.randomUUID?.() ?? `card-${Date.now()}-${Math.random().toString(16).slice(2)}`;
}

function defaultCard(index = 0) {
  return {
    id: uid(),
    title: `Card ${index + 1}`,
    description: 'Description',
    badge: String(index + 1),
    badgeSubtitle: '',
    accent: '#d8172d',
    background: '#27495f',
    artwork: null,
    artworkName: null,
  };
}

function showToast(message) {
  const toast = $('#toast');
  toast.textContent = message;
  toast.classList.add('show');
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => toast.classList.remove('show'), 2600);
}

function persistProject() {
  try {
    localStorage.setItem('watchcompare.project.v1', JSON.stringify(state.project));
  } catch (error) {
    console.warn('Could not persist project', error);
  }
}

function restoreProject() {
  try {
    const raw = localStorage.getItem('watchcompare.project.v1');
    if (!raw) return;
    const parsed = JSON.parse(raw);
    if (!parsed || !Array.isArray(parsed.cards)) return;
    state.project = normalizeProject(parsed);
    state.selectedId = state.project.cards[0]?.id ?? null;
  } catch (error) {
    console.warn('Could not restore project', error);
  }
}

function normalizeProject(project) {
  const cards = Array.isArray(project.cards) ? project.cards.map((card, index) => ({
    ...defaultCard(index),
    ...card,
    id: card.id || uid(),
    artwork: card.artwork || null,
    artworkName: card.artworkName || null,
  })) : [];
  return {
    version: 1,
    name: String(project.name || 'Untitled comparison'),
    cards,
  };
}

function currentCard() {
  return state.project.cards.find((card) => card.id === state.selectedId) || null;
}

function selectCard(id) {
  state.selectedId = id;
  renderCardList();
  syncInspector();
  renderCurrentFrame();
}

function renderCardList() {
  const list = $('#cardList');
  list.textContent = '';
  $('#cardCount').textContent = `${state.project.cards.length} ${state.project.cards.length === 1 ? 'card' : 'cards'}`;

  state.project.cards.forEach((card, index) => {
    const row = document.createElement('div');
    row.className = `card-row${card.id === state.selectedId ? ' selected' : ''}`;
    row.addEventListener('click', () => selectCard(card.id));

    const thumb = document.createElement('div');
    thumb.className = 'card-thumb';
    thumb.style.background = card.background;
    if (card.artwork) {
      const img = document.createElement('img');
      img.src = card.artwork;
      img.alt = '';
      thumb.appendChild(img);
    } else {
      thumb.textContent = card.badge || String(index + 1);
    }

    const copy = document.createElement('div');
    const title = document.createElement('strong');
    title.textContent = card.title || `Card ${index + 1}`;
    const desc = document.createElement('span');
    desc.textContent = card.description || 'No description';
    copy.append(title, desc);

    const idx = document.createElement('div');
    idx.className = 'card-index';
    idx.textContent = String(index + 1).padStart(2, '0');

    row.append(thumb, copy, idx);
    list.appendChild(row);
  });
}

function syncInspector() {
  const card = currentCard();
  $('#inspectorEmpty').hidden = Boolean(card);
  $('#cardForm').hidden = !card;
  if (!card) return;

  const index = state.project.cards.findIndex((item) => item.id === card.id);
  $('#inspectorTitle').textContent = `Card ${index + 1}`;
  $('#cardTitle').value = card.title;
  $('#cardDescription').value = card.description;
  $('#cardBadge').value = card.badge;
  $('#cardBadgeSubtitle').value = card.badgeSubtitle;
  $('#cardAccent').value = card.accent;
  $('#cardBackground').value = card.background;
  $('#artworkName').textContent = card.artworkName || 'No image';
  $('#moveUpButton').disabled = index <= 0;
  $('#moveDownButton').disabled = index < 0 || index >= state.project.cards.length - 1;
}

function commitInspector() {
  const card = currentCard();
  if (!card) return;
  card.title = $('#cardTitle').value;
  card.description = $('#cardDescription').value;
  card.badge = $('#cardBadge').value;
  card.badgeSubtitle = $('#cardBadgeSubtitle').value;
  card.accent = $('#cardAccent').value;
  card.background = $('#cardBackground').value;
  persistProject();
  renderCardList();
  renderCurrentFrame();
}

function addCard() {
  const card = defaultCard(state.project.cards.length);
  state.project.cards.push(card);
  state.selectedId = card.id;
  persistProject();
  renderCardList();
  syncInspector();
  renderCurrentFrame();
}

function moveCard(delta) {
  const index = state.project.cards.findIndex((card) => card.id === state.selectedId);
  const target = index + delta;
  if (index < 0 || target < 0 || target >= state.project.cards.length) return;
  const [card] = state.project.cards.splice(index, 1);
  state.project.cards.splice(target, 0, card);
  persistProject();
  renderCardList();
  syncInspector();
  renderCurrentFrame();
}

function duplicateCard() {
  const card = currentCard();
  if (!card) return;
  const index = state.project.cards.findIndex((item) => item.id === card.id);
  const duplicate = { ...card, id: uid(), title: `${card.title} copy` };
  state.project.cards.splice(index + 1, 0, duplicate);
  state.selectedId = duplicate.id;
  persistProject();
  renderCardList();
  syncInspector();
  renderCurrentFrame();
}

function deleteCard() {
  const index = state.project.cards.findIndex((card) => card.id === state.selectedId);
  if (index < 0) return;
  state.project.cards.splice(index, 1);
  state.selectedId = state.project.cards[Math.min(index, state.project.cards.length - 1)]?.id ?? null;
  persistProject();
  renderCardList();
  syncInspector();
  renderCurrentFrame();
}

function parseCsv(text) {
  const rows = [];
  let row = [];
  let field = '';
  let quoted = false;
  for (let i = 0; i < text.length; i += 1) {
    const ch = text[i];
    if (quoted) {
      if (ch === '"' && text[i + 1] === '"') {
        field += '"';
        i += 1;
      } else if (ch === '"') {
        quoted = false;
      } else {
        field += ch;
      }
    } else if (ch === '"') {
      quoted = true;
    } else if (ch === ',') {
      row.push(field);
      field = '';
    } else if (ch === '\n') {
      row.push(field.replace(/\r$/, ''));
      rows.push(row);
      row = [];
      field = '';
    } else {
      field += ch;
    }
  }
  row.push(field.replace(/\r$/, ''));
  if (row.some((value) => value.length)) rows.push(row);
  return rows;
}

function importCsvText(text) {
  const rows = parseCsv(text);
  if (rows.length < 2) throw new Error('CSV needs a header row and at least one data row.');
  const headers = rows[0].map((value) => value.trim().toLowerCase().replace(/[\s_-]+/g, ''));
  const find = (...names) => names.map((name) => headers.indexOf(name)).find((index) => index >= 0) ?? -1;
  const titleIndex = find('title', 'name');
  const descriptionIndex = find('description', 'desc', 'subtitle');
  const badgeIndex = find('badge', 'value', 'year', 'rank');
  const badgeSubtitleIndex = find('badgesubtitle', 'units', 'unit');
  const backgroundIndex = find('background', 'backgroundcolor', 'bg');
  const accentIndex = find('accent', 'accentcolor', 'badgecolor');

  const cards = rows.slice(1).filter((row) => row.some((value) => value.trim())).map((row, index) => {
    const card = defaultCard(index);
    if (titleIndex >= 0) card.title = row[titleIndex]?.trim() || card.title;
    if (descriptionIndex >= 0) card.description = row[descriptionIndex]?.trim() || '';
    if (badgeIndex >= 0) card.badge = row[badgeIndex]?.trim() || '';
    if (badgeSubtitleIndex >= 0) card.badgeSubtitle = row[badgeSubtitleIndex]?.trim() || '';
    if (backgroundIndex >= 0 && /^#[0-9a-f]{6}$/i.test(row[backgroundIndex]?.trim())) card.background = row[backgroundIndex].trim();
    if (accentIndex >= 0 && /^#[0-9a-f]{6}$/i.test(row[accentIndex]?.trim())) card.accent = row[accentIndex].trim();
    return card;
  });
  state.project.cards = cards;
  state.selectedId = cards[0]?.id ?? null;
  persistProject();
  renderCardList();
  syncInspector();
  renderCurrentFrame();
  showToast(`Imported ${cards.length} cards from CSV.`);
}

function downloadBlob(blob, filename) {
  const file = new File([blob], filename, { type: blob.type || 'application/octet-stream' });
  if (navigator.canShare?.({ files: [file] })) {
    navigator.share({ files: [file], title: filename }).catch((error) => {
      if (error?.name !== 'AbortError') fallbackDownload(blob, filename);
    });
    return;
  }
  fallbackDownload(blob, filename);
}

function fallbackDownload(blob, filename) {
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  a.remove();
  setTimeout(() => URL.revokeObjectURL(url), 1500);
}

function safeFilename(value) {
  return String(value || 'watchcompare').trim().replace(/[^a-z0-9._-]+/gi, '-').replace(/^-+|-+$/g, '') || 'watchcompare';
}

function getFrameState(frame) {
  const exact = state.track[frame];
  if (exact) return exact;
  const timeSeconds = frame / 60;
  return {
    frame,
    time_seconds: timeSeconds,
    time_millis: timeSeconds * 1000,
    stage: frame < 630 ? 'intro' : frame < 11843 ? 'cruise' : frame < 12180 ? 'outro' : 'fade',
    card_train_x_px: frame < 630 ? 0 : -(frame - 630) * (133.47312789378643 / 60) - 313.5,
    card_phase_px: 0,
  };
}

function requestDetailedScene(frame) {
  const needsDetail = frame < 430 || frame >= 11868;
  if (!invoke || !needsDetail) {
    state.scene = null;
    return;
  }
  const requestId = ++state.sceneRequestId;
  invoke('reference_scene', { frame }).then((scene) => {
    if (requestId !== state.sceneRequestId || state.frame !== frame) return;
    state.scene = scene;
    renderCurrentFrame();
  }).catch((error) => console.warn('reference_scene unavailable', error));
}

function setFrame(frame, requestScene = true) {
  const max = state.profile.frame_count - 1;
  state.frame = Math.max(0, Math.min(max, Math.round(frame)));
  $('#timeline').value = String(state.frame);
  const frameState = getFrameState(state.frame);
  const millis = frameState.time_millis ?? (state.frame / state.profile.fps) * 1000;
  $('#frameLabel').textContent = `Frame ${state.frame.toLocaleString()} · ${millis.toFixed(3)} ms`;
  $('#timeReadout').textContent = formatTime(millis);
  $('#stageReadout').textContent = frameState.stage;
  $('#trainX').textContent = `${frameState.card_train_x_px.toFixed(1)} px`;
  $('#pitchReadout').textContent = `${state.profile.geometry.card_pitch_px} px`;
  if (requestScene) requestDetailedScene(state.frame);
  renderCurrentFrame();
}

function formatTime(ms) {
  const total = Math.max(0, ms);
  const minutes = Math.floor(total / 60000);
  const seconds = Math.floor((total % 60000) / 1000);
  const millis = Math.floor(total % 1000);
  return `${String(minutes).padStart(2, '0')}:${String(seconds).padStart(2, '0')}.${String(millis).padStart(3, '0')}`;
}

function ensureImage(src) {
  if (!src) return null;
  const cached = imageCache.get(src);
  if (cached) return cached.loaded ? cached.image : null;
  const image = new Image();
  const record = { image, loaded: false };
  imageCache.set(src, record);
  image.onload = () => {
    record.loaded = true;
    renderCurrentFrame();
  };
  image.onerror = () => imageCache.delete(src);
  image.src = src;
  return null;
}

function drawImageCover(image, x, y, width, height) {
  const sourceRatio = image.width / image.height;
  const targetRatio = width / height;
  let sx = 0, sy = 0, sw = image.width, sh = image.height;
  if (sourceRatio > targetRatio) {
    sw = image.height * targetRatio;
    sx = (image.width - sw) / 2;
  } else {
    sh = image.width / targetRatio;
    sy = (image.height - sh) / 2;
  }
  ctx.drawImage(image, sx, sy, sw, sh, x, y, width, height);
}

function drawBadge(card, x, cardWidth, index) {
  const baseWidth = 246;
  const baseHeight = 282;
  let scale = 1;
  let offsetX = 0;
  let offsetY = 0;
  if (index === 1 && state.scene?.second_badge_transform) {
    scale = state.scene.second_badge_transform.scale;
    offsetX = state.scene.second_badge_transform.x;
    offsetY = state.scene.second_badge_transform.y;
  }
  const width = baseWidth * scale;
  const height = baseHeight * scale;
  const bx = x + (cardWidth - width) / 2 + offsetX;
  const by = 68 + offsetY;

  ctx.save();
  ctx.beginPath();
  ctx.moveTo(bx + width * 0.5, by);
  ctx.lineTo(bx + width, by + height * 0.245);
  ctx.lineTo(bx + width, by + height * 0.75);
  ctx.lineTo(bx + width * 0.505, by + height);
  ctx.lineTo(bx, by + height * 0.75);
  ctx.lineTo(bx, by + height * 0.245);
  ctx.closePath();
  ctx.fillStyle = card.accent || '#d8172d';
  ctx.fill();
  ctx.clip();

  if (index === 1 && state.scene?.second_badge_shine) {
    const shine = state.scene.second_badge_shine;
    ctx.save();
    ctx.translate(bx + width / 2, by + height / 2);
    ctx.rotate((120 * Math.PI) / 180);
    const bandWidth = Math.max(14, shine.width80_px * 1.6);
    const center = (shine.normal_center_px - 250) * 0.75;
    const gradient = ctx.createLinearGradient(center - bandWidth, 0, center + bandWidth, 0);
    gradient.addColorStop(0, 'rgba(255,255,255,0)');
    gradient.addColorStop(.48, 'rgba(255,255,255,.1)');
    gradient.addColorStop(.5, 'rgba(255,255,255,.75)');
    gradient.addColorStop(.52, 'rgba(255,255,255,.1)');
    gradient.addColorStop(1, 'rgba(255,255,255,0)');
    ctx.fillStyle = gradient;
    ctx.fillRect(center - bandWidth, -height, bandWidth * 2, height * 2);
    ctx.restore();
  }

  ctx.fillStyle = '#fff';
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  ctx.font = `900 ${Math.max(28, 49 * scale)}px "WatchCompareUserFont", Arial, sans-serif`;
  ctx.fillText(card.badge || '', bx + width / 2, by + height * 0.42, width * 0.84);
  if (card.badgeSubtitle) {
    ctx.font = `900 ${Math.max(13, 20 * scale)}px "WatchCompareUserFont", Arial, sans-serif`;
    ctx.fillText(card.badgeSubtitle, bx + width / 2, by + height * 0.59, width * 0.82);
  }
  ctx.restore();
}

function drawCard(card, x, index) {
  const g = state.profile.geometry;
  const pitch = g.card_pitch_px;
  const separator = g.separator_nominal_px;
  const width = pitch - separator;
  const artHeight = g.artwork_bottom_y + 1;

  if (x + pitch < 0 || x > canvas.width) return;

  ctx.save();
  ctx.beginPath();
  ctx.rect(x, 0, width, canvas.height);
  ctx.clip();

  ctx.fillStyle = card.background || '#27495f';
  ctx.fillRect(x, 0, width, artHeight);
  const image = ensureImage(card.artwork);
  if (image) {
    drawImageCover(image, x, 0, width, artHeight);
  } else {
    const gradient = ctx.createLinearGradient(x, 0, x + width, artHeight);
    gradient.addColorStop(0, 'rgba(255,255,255,.04)');
    gradient.addColorStop(.5, 'rgba(0,0,0,0)');
    gradient.addColorStop(1, 'rgba(0,0,0,.24)');
    ctx.fillStyle = gradient;
    ctx.fillRect(x, 0, width, artHeight);
  }

  drawBadge(card, x, width, index);

  ctx.fillStyle = '#f2f2f2';
  ctx.fillRect(x, g.title_top_y, width, g.title_bottom_y - g.title_top_y + 1);
  ctx.fillStyle = '#111';
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  ctx.font = '900 40px "WatchCompareUserFont", Arial, sans-serif';
  ctx.fillText(card.title || '', x + width / 2, (g.title_top_y + g.title_bottom_y) / 2 + 1, width - 28);

  ctx.fillStyle = '#605f5b';
  ctx.fillRect(x, g.description_top_y, width, g.description_bottom_y - g.description_top_y + 1);
  ctx.fillStyle = '#fff';
  ctx.font = '900 28px "WatchCompareUserFont", Arial, sans-serif';
  ctx.fillText(card.description || '', x + width / 2, (g.description_top_y + g.description_bottom_y) / 2 + 1, width - 30);

  ctx.fillStyle = '#101010';
  ctx.fillRect(x, g.bottom_border_top_y, width, canvas.height - g.bottom_border_top_y);
  ctx.restore();

  ctx.fillStyle = '#101010';
  ctx.fillRect(x + width, 0, separator, canvas.height);
}

function drawCreditsOverlay() {
  const left = state.scene?.credits_left_x_px;
  if (left == null || left >= canvas.width) return;
  ctx.save();
  ctx.fillStyle = '#101010';
  ctx.fillRect(left, 0, canvas.width - left, canvas.height);
  ctx.fillStyle = '#d7d7d7';
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  ctx.font = '700 30px "WatchCompareUserFont", Arial, sans-serif';
  ctx.fillText('WATCHCOMPARE', left + (canvas.width - left) / 2, canvas.height / 2);
  ctx.restore();
}

function drawOutro() {
  const scene = state.scene;
  if (!scene) return;

  if (scene.outro_wipe_bottom_y != null) {
    ctx.fillStyle = '#050505';
    ctx.fillRect(0, 0, canvas.width, scene.outro_wipe_bottom_y + 1);
  }

  if (scene.outro_group) {
    const top = scene.outro_group.panel_top_y;
    ctx.fillStyle = '#151515';
    ctx.fillRect(70, top, 1240, 430);
    ctx.fillStyle = '#e92735';
    ctx.fillRect(112, top + 58, 510, 286);
    ctx.fillRect(758, top + 58, 510, 286);
    ctx.fillStyle = '#fff';
    ctx.textAlign = 'center';
    ctx.font = '900 28px "WatchCompareUserFont", Arial, sans-serif';
    ctx.fillText('BEST VIDEO FOR YOU', 367, top + 374);
    ctx.fillText('NEWEST VIDEO', 1013, top + 374);
    ctx.fillStyle = '#888';
    ctx.font = '900 25px "WatchCompareUserFont", Arial, sans-serif';
    ctx.fillText('Video Made By', 690, scene.outro_group.credits_top_y);
  }

  if (scene.outro_cta_bbox) {
    const box = scene.outro_cta_bbox;
    ctx.fillStyle = '#f5f5f5';
    ctx.fillRect(box.x, box.y, box.width, box.height);
    if (box.width > 250 && box.height > 70) {
      ctx.fillStyle = scene.cta_like_blue_level ? `rgba(30,120,255,${Math.max(.25, scene.cta_like_blue_level)})` : '#252525';
      ctx.fillRect(box.x + 30, box.y + 30, 72, 48);
      ctx.fillStyle = '#e6212d';
      ctx.fillRect(box.x + box.width / 2 - 90, box.y + 25, 180, 58);
      ctx.fillStyle = '#fff';
      ctx.textAlign = 'center';
      ctx.textBaseline = 'middle';
      ctx.font = '900 20px Arial, sans-serif';
      ctx.fillText(scene.cta_subscribed_bbox ? 'SUBSCRIBED' : 'SUBSCRIBE', box.x + box.width / 2, box.y + 54);
    }
  }
}

function renderCurrentFrame() {
  ctx.save();
  ctx.fillStyle = '#101010';
  ctx.fillRect(0, 0, canvas.width, canvas.height);

  const cards = state.project.cards;
  emptyCanvasState.hidden = cards.length > 0;
  if (cards.length) {
    const frameState = getFrameState(state.frame);
    const pitch = state.profile.geometry.card_pitch_px;
    let trainX = frameState.card_train_x_px;

    if (state.frame < 91 && cards[0]) {
      const reveal = state.scene?.first_card_reveal_width_px ?? Math.min(pitch, (state.frame / 90) * pitch);
      ctx.save();
      ctx.beginPath();
      ctx.rect(0, 0, reveal, canvas.height);
      ctx.clip();
      drawCard(cards[0], 0, 0);
      ctx.restore();
      for (let i = 1; i < cards.length; i += 1) drawCard(cards[i], i * pitch, i);
    } else {
      cards.forEach((card, index) => drawCard(card, index * pitch + trainX, index));
    }

    drawCreditsOverlay();
    if (state.frame >= 11868) drawOutro();
  }

  const fade = state.scene?.outro_fade_level;
  if (typeof fade === 'number' && fade < 1) {
    ctx.fillStyle = `rgba(0,0,0,${1 - fade})`;
    ctx.fillRect(0, 0, canvas.width, canvas.height);
  }
  ctx.restore();

  $('#creditsReadout').textContent = state.scene?.credits_left_x_px != null ? `${state.scene.credits_left_x_px} px` : '—';
  $('#fadeReadout').textContent = `${Math.round((state.scene?.outro_fade_level ?? 1) * 100)}%`;
}

function applyZoom() {
  let scale = 1;
  if (state.zoom === 'fit') {
    const availableWidth = Math.max(240, stageScroller.clientWidth - 36);
    const availableHeight = Math.max(140, stageScroller.clientHeight - 36);
    scale = Math.min(availableWidth / canvas.width, availableHeight / canvas.height);
  } else {
    scale = Number(state.zoom);
  }
  scale = Math.max(.12, Math.min(1, scale));
  canvasWrap.style.width = `${canvas.width * scale}px`;
  canvasWrap.style.height = `${canvas.height * scale}px`;
  document.querySelectorAll('.zoom-tools button').forEach((button) => {
    button.classList.toggle('active', button.dataset.zoom === String(state.zoom));
  });
}

function togglePlayback() {
  state.playing = !state.playing;
  $('#playButton').textContent = state.playing ? '❚❚' : '▶';
  state.lastPlaybackTime = 0;
  state.frameAccumulator = 0;
  if (state.playing) requestAnimationFrame(playbackTick);
}

function playbackTick(timestamp) {
  if (!state.playing) return;
  if (!state.lastPlaybackTime) state.lastPlaybackTime = timestamp;
  const elapsed = timestamp - state.lastPlaybackTime;
  state.lastPlaybackTime = timestamp;
  state.frameAccumulator += (elapsed * state.profile.fps) / 1000;
  const advance = Math.floor(state.frameAccumulator);
  if (advance > 0) {
    state.frameAccumulator -= advance;
    const next = state.frame + advance;
    if (next >= state.profile.frame_count - 1) {
      state.playing = false;
      $('#playButton').textContent = '▶';
      setFrame(state.profile.frame_count - 1);
      return;
    }
    setFrame(next, false);
    if (next < 430 || next >= 11868) requestDetailedScene(next);
  }
  requestAnimationFrame(playbackTick);
}

async function loadReferenceModel() {
  if (!invoke) {
    $('#bridgeStatus').textContent = 'Web preview · Rust bridge unavailable';
    setFrame(0);
    return;
  }
  try {
    const [profile, track] = await Promise.all([
      invoke('reference_profile'),
      invoke('reference_track'),
    ]);
    state.profile = profile;
    state.track = track;
    $('#timeline').max = String(profile.frame_count - 1);
    $('#bridgeStatus').textContent = `Rust timeline loaded · ${profile.frame_count.toLocaleString()} exact samples`;
    setFrame(Math.min(state.frame, profile.frame_count - 1));
    applyZoom();
  } catch (error) {
    console.error(error);
    $('#bridgeStatus').textContent = 'Rust bridge error · fallback preview active';
    showToast('Could not load the measured Rust timeline; using fallback motion.');
  }
}

async function loadFonts(files) {
  let loaded = 0;
  for (const file of files) {
    try {
      const name = file.name.toLowerCase();
      const weight = name.includes('heavy') ? '900' : name.includes('bold') ? '700' : name.includes('medium') ? '500' : '400';
      const face = new FontFace('WatchCompareUserFont', await file.arrayBuffer(), { weight });
      await face.load();
      document.fonts.add(face);
      loaded += 1;
    } catch (error) {
      console.warn('Font load failed', file.name, error);
    }
  }
  if (loaded) {
    showToast(`Loaded ${loaded} local font ${loaded === 1 ? 'file' : 'files'}.`);
    renderCurrentFrame();
  } else {
    showToast('No font files could be loaded.');
  }
}

function bindEvents() {
  $('#projectName').addEventListener('input', (event) => {
    state.project.name = event.target.value;
    persistProject();
  });

  $('#addCardButton').addEventListener('click', addCard);
  $('#newProjectButton').addEventListener('click', () => {
    if (state.project.cards.length && !confirm('Create a new project? Unsaved exported files are not affected.')) return;
    state.project = { version: 1, name: 'Untitled comparison', cards: [] };
    state.selectedId = null;
    $('#projectName').value = state.project.name;
    persistProject();
    renderCardList();
    syncInspector();
    setFrame(0);
  });

  $('#importProjectButton').addEventListener('click', () => $('#projectFileInput').click());
  $('#importCsvButton').addEventListener('click', () => $('#csvFileInput').click());
  $('#loadFontButton').addEventListener('click', () => $('#fontFileInput').click());
  $('#chooseArtworkButton').addEventListener('click', () => $('#artworkFileInput').click());

  $('#projectFileInput').addEventListener('change', async (event) => {
    const file = event.target.files?.[0];
    event.target.value = '';
    if (!file) return;
    try {
      const project = normalizeProject(JSON.parse(await file.text()));
      state.project = project;
      state.selectedId = project.cards[0]?.id ?? null;
      $('#projectName').value = project.name;
      persistProject();
      renderCardList();
      syncInspector();
      renderCurrentFrame();
      showToast(`Imported ${project.cards.length} cards.`);
    } catch (error) {
      showToast(`Project import failed: ${error.message}`);
    }
  });

  $('#csvFileInput').addEventListener('change', async (event) => {
    const file = event.target.files?.[0];
    event.target.value = '';
    if (!file) return;
    try {
      importCsvText(await file.text());
    } catch (error) {
      showToast(`CSV import failed: ${error.message}`);
    }
  });

  $('#fontFileInput').addEventListener('change', (event) => {
    const files = [...(event.target.files || [])];
    event.target.value = '';
    if (files.length) loadFonts(files);
  });

  $('#artworkFileInput').addEventListener('change', async (event) => {
    const file = event.target.files?.[0];
    event.target.value = '';
    const card = currentCard();
    if (!file || !card) return;
    const reader = new FileReader();
    reader.onload = () => {
      card.artwork = reader.result;
      card.artworkName = file.name;
      imageCache.delete(card.artwork);
      persistProject();
      syncInspector();
      renderCardList();
      renderCurrentFrame();
    };
    reader.readAsDataURL(file);
  });

  $('#exportProjectButton').addEventListener('click', () => {
    const blob = new Blob([JSON.stringify(state.project, null, 2)], { type: 'application/json' });
    downloadBlob(blob, `${safeFilename(state.project.name)}.watchcompare.json`);
  });

  $('#exportFrameButton').addEventListener('click', () => {
    if (!state.project.cards.length) {
      showToast('Add or import cards before exporting a frame.');
      return;
    }
    canvas.toBlob((blob) => {
      if (!blob) return showToast('Could not encode the PNG frame.');
      downloadBlob(blob, `${safeFilename(state.project.name)}-frame-${String(state.frame).padStart(5, '0')}.png`);
    }, 'image/png');
  });

  ['cardTitle', 'cardDescription', 'cardBadge', 'cardBadgeSubtitle', 'cardAccent', 'cardBackground'].forEach((id) => {
    $(`#${id}`).addEventListener('input', commitInspector);
  });

  $('#moveUpButton').addEventListener('click', () => moveCard(-1));
  $('#moveDownButton').addEventListener('click', () => moveCard(1));
  $('#duplicateCardButton').addEventListener('click', duplicateCard);
  $('#deleteCardButton').addEventListener('click', deleteCard);

  $('#timeline').addEventListener('input', (event) => setFrame(Number(event.target.value)));
  $('#playButton').addEventListener('click', togglePlayback);
  $('#firstFrameButton').addEventListener('click', () => setFrame(0));
  $('#prevFrameButton').addEventListener('click', () => setFrame(state.frame - 1));
  $('#nextFrameButton').addEventListener('click', () => setFrame(state.frame + 1));
  $('#lastFrameButton').addEventListener('click', () => setFrame(state.profile.frame_count - 1));

  document.querySelectorAll('.zoom-tools button').forEach((button) => {
    button.addEventListener('click', () => {
      state.zoom = button.dataset.zoom;
      applyZoom();
    });
  });

  window.addEventListener('resize', () => {
    if (state.zoom === 'fit') applyZoom();
  });

  window.addEventListener('keydown', (event) => {
    if (event.target.matches('input,textarea')) return;
    if (event.code === 'Space') {
      event.preventDefault();
      togglePlayback();
    } else if (event.code === 'ArrowLeft') {
      event.preventDefault();
      setFrame(state.frame - 1);
    } else if (event.code === 'ArrowRight') {
      event.preventDefault();
      setFrame(state.frame + 1);
    }
  });
}

restoreProject();
$('#projectName').value = state.project.name;
renderCardList();
syncInspector();
bindEvents();
setFrame(0);
requestAnimationFrame(applyZoom);
loadReferenceModel();
