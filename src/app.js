const $ = (selector) => document.querySelector(selector);
const $$ = (selector) => [...document.querySelectorAll(selector)];
const invoke = window.__TAURI__?.core?.invoke;

let project = null;
let selectedCard = 0;
let currentFrame = 0;
let playing = false;
let playEpoch = 0;
let playStartFrame = 0;
let previewTimer = null;
let previewBusy = false;
let previewQueued = false;
let exportJobId = null;
let exportPoll = null;
let dirty = false;
let referenceProfile = null;
let platform = null;

function setBackend(text) { $('#backendState').textContent = text; }
function setDirty(value = true) {
  dirty = value;
  $('#dirtyState').textContent = value ? 'Unsaved changes' : 'Saved';
  $('#dirtyState').classList.toggle('dirty', value);
}

function hexByte(value) { return Number(value).toString(16).padStart(2, '0'); }
function rgbaToHex(value) {
  const [r = 0, g = 0, b = 0] = value || [];
  return `#${hexByte(r)}${hexByte(g)}${hexByte(b)}`;
}
function hexToRgba(hex, alpha = 255) {
  const value = /^#[0-9a-f]{6}$/i.test(hex) ? hex : '#000000';
  return [parseInt(value.slice(1, 3), 16), parseInt(value.slice(3, 5), 16), parseInt(value.slice(5, 7), 16), alpha];
}
function clone(value) { return JSON.parse(JSON.stringify(value)); }
function clamp(value, min, max) { return Math.max(min, Math.min(max, value)); }
function numberValue(selector, fallback) {
  const value = Number($(selector).value);
  return Number.isFinite(value) ? value : fallback;
}
function durationFrames() {
  if (!project) return 1;
  return Math.max(1, Math.round(project.export.duration_seconds * project.export.fps));
}
function selected() { return project?.cards?.[selectedCard] || null; }

function formatTime(frame) {
  const fps = Math.max(1, project?.export?.fps || 60);
  const seconds = frame / fps;
  const whole = Math.floor(seconds);
  const minutes = Math.floor(whole / 60);
  const rem = whole % 60;
  const millis = Math.floor((seconds - whole) * 1000);
  return `${String(minutes).padStart(2, '0')}:${String(rem).padStart(2, '0')}.${String(millis).padStart(3, '0')}`;
}

function updateTransport() {
  const max = Math.max(0, durationFrames() - 1);
  currentFrame = clamp(currentFrame, 0, max);
  $('#frameSlider').max = String(max);
  $('#frameSlider').value = String(currentFrame);
  $('#timeLabel').textContent = formatTime(currentFrame);
  $('#frameLabel').textContent = `frame ${currentFrame.toLocaleString()}`;
  $('#debugState').textContent = `frame ${currentFrame} / ${max}`;
  $('#playPause').textContent = playing ? '❚❚' : '▶';
}

function renderCardList() {
  $('#cardCount').textContent = `${project.cards.length} card${project.cards.length === 1 ? '' : 's'}`;
  const list = $('#cardList');
  list.innerHTML = '';
  project.cards.forEach((card, index) => {
    const button = document.createElement('button');
    button.className = `card-row${index === selectedCard ? ' selected' : ''}`;
    button.innerHTML = `
      <span class="card-index">${String(index + 1).padStart(2, '0')}</span>
      <span class="card-swatch" style="background:${rgbaToHex(card.artwork_color)}"></span>
      <span class="card-copy"><strong>${escapeHtml(card.title || 'Untitled')}</strong><small>${escapeHtml(card.badge_value || 'No badge')}</small></span>
    `;
    button.addEventListener('click', () => {
      selectedCard = index;
      renderCardList();
      populateCardInspector();
    });
    list.appendChild(button);
  });
}

function escapeHtml(value) {
  return String(value).replace(/[&<>'"]/g, ch => ({'&':'&amp;','<':'&lt;','>':'&gt;',"'":'&#39;','"':'&quot;'}[ch]));
}

function populateCardInspector() {
  const card = selected();
  if (!card) return;
  $('#cardTitle').value = card.title || '';
  $('#cardDescription').value = card.description || '';
  $('#badgeValue').value = card.badge_value || '';
  $('#badgeLabel').value = card.badge_label || '';
  $('#artworkPath').value = card.artwork_path || '';
  const color = rgbaToHex(card.artwork_color);
  $('#artworkColor').value = color;
  $('#artworkColorHex').value = color.toUpperCase();
}

function populateProjectInspector() {
  $('#projectName').value = project.name;
  $('#modelKind').value = project.model;
  $('#exportWidth').value = project.export.width;
  $('#exportHeight').value = project.export.height;
  $('#exportFps').value = project.export.fps;
  $('#durationSeconds').value = project.export.duration_seconds;
  $('#cardPitch').value = project.theme.card_pitch_px;
  $('#scrollSpeed').value = project.theme.scroll_px_per_second;
  $('#fontPath').value = project.font_path || '';
  $('#soundtrackPath').value = project.export.soundtrack_path || '';
  $('#videoBitrate').value = project.export.video_bitrate_mbps;
  $('#audioBitrate').value = project.export.audio_bitrate_kbps;
  $('#resolutionLabel').textContent = `${project.export.width}×${project.export.height}`;
  $('#fpsLabel').textContent = `${project.export.fps} FPS`;
  updateTransport();
}

async function suggestPaths() {
  if (!invoke || !project) return;
  try {
    const [projectPath, outputPath] = await Promise.all([
      invoke('suggested_project_path', { project: clone(project) }),
      invoke('suggested_export_path', { project: clone(project) }),
    ]);
    if (!$('#projectSavePath').dataset.edited) $('#projectSavePath').value = projectPath;
    if (!$('#outputPath').dataset.edited) $('#outputPath').value = outputPath;
  } catch (error) {
    console.warn('suggest paths failed', error);
  }
}

function populateAll() {
  selectedCard = clamp(selectedCard, 0, Math.max(0, project.cards.length - 1));
  renderCardList();
  populateCardInspector();
  populateProjectInspector();
  suggestPaths();
  queuePreview(0);
}

function mutateCard(mutator) {
  const card = selected();
  if (!card) return;
  mutator(card);
  setDirty(true);
  renderCardList();
  queuePreview();
}

function mutateProject(mutator, refreshPaths = false) {
  mutator(project);
  setDirty(true);
  populateProjectInspector();
  if (refreshPaths) suggestPaths();
  queuePreview();
}

function queuePreview(delay = 80) {
  clearTimeout(previewTimer);
  previewTimer = setTimeout(renderPreview, delay);
}

async function renderPreview() {
  if (!invoke || !project) return;
  if (previewBusy) {
    previewQueued = true;
    return;
  }
  previewBusy = true;
  previewQueued = false;
  $('#renderStatus').textContent = 'rendering…';
  try {
    const dataUrl = await invoke('render_preview', { project: clone(project), frame: currentFrame });
    $('#previewImage').src = dataUrl;
    $('#previewImage').style.display = 'block';
    $('#previewEmpty').style.display = 'none';
    $('#renderStatus').textContent = 'frame ready';
    setBackend('Rust renderer connected');
  } catch (error) {
    $('#renderStatus').textContent = 'render failed';
    $('#previewEmpty').style.display = 'grid';
    $('#previewEmpty').textContent = String(error);
    setBackend(`Renderer error: ${error}`);
  } finally {
    previewBusy = false;
    if (previewQueued) queuePreview(0);
  }
}

function setFrame(frame, immediate = false) {
  currentFrame = Math.round(clamp(Number(frame) || 0, 0, Math.max(0, durationFrames() - 1)));
  updateTransport();
  queuePreview(immediate ? 0 : 35);
}

function startPlayback() {
  if (playing) return;
  playing = true;
  playEpoch = performance.now();
  playStartFrame = currentFrame;
  updateTransport();
  requestAnimationFrame(playTick);
}

function stopPlayback() {
  playing = false;
  updateTransport();
}

function playTick(now) {
  if (!playing || !project) return;
  const elapsed = (now - playEpoch) / 1000;
  const next = playStartFrame + Math.floor(elapsed * project.export.fps);
  if (next >= durationFrames()) {
    setFrame(durationFrames() - 1, true);
    stopPlayback();
    return;
  }
  if (next !== currentFrame) setFrame(next, true);
  requestAnimationFrame(playTick);
}

async function sampleReferenceFrame() {
  if (!invoke) return;
  try {
    const state = await invoke('sample_reference_scene_state', { frame: currentFrame });
    const mid = state.mid_video_cta?.visible ? ` · mid CTA ${state.mid_video_cta.phase}` : '';
    $('#refScene').textContent = `${state.stage}${mid}`;
    $('#debugState').textContent = `ref ${state.frame} · ${state.time_millis.toFixed(3)} ms · x ${state.card_train_x_px.toFixed(1)}`;
  } catch (error) {
    $('#refScene').textContent = 'unavailable';
  }
}

async function createNewProject() {
  try {
    project = invoke ? await invoke('new_project') : null;
    if (!project) throw new Error('Tauri backend unavailable');
    selectedCard = 0;
    currentFrame = 0;
    setDirty(false);
    $('#projectSavePath').dataset.edited = '';
    $('#outputPath').dataset.edited = '';
    populateAll();
  } catch (error) {
    setBackend(String(error));
  }
}

async function validateIncoming(candidate) {
  if (invoke) await invoke('validate_project', { project: clone(candidate) });
  return candidate;
}

async function importProjectFile(file) {
  try {
    const candidate = JSON.parse(await file.text());
    project = await validateIncoming(candidate);
    selectedCard = 0;
    currentFrame = 0;
    setDirty(false);
    populateAll();
    setBackend(`Imported ${file.name}`);
  } catch (error) {
    setBackend(`Import failed: ${error}`);
  }
}

function downloadProjectJson() {
  if (!project) return;
  const blob = new Blob([JSON.stringify(project, null, 2)], { type: 'application/json' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = `${project.name.replace(/[^a-z0-9_-]+/gi, '-') || 'watchcompare'}.watchcompare.json`;
  a.click();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}

async function saveProject() {
  if (!invoke || !project) return;
  const path = $('#projectSavePath').value.trim();
  if (!path) return setBackend('Choose a project save path in Export settings');
  try {
    await invoke('save_project', { path, project: clone(project) });
    setDirty(false);
    setBackend(`Saved ${path}`);
  } catch (error) {
    setBackend(`Save failed: ${error}`);
  }
}

async function startExport() {
  if (!invoke || !project || exportJobId) return;
  const outputPath = $('#outputPath').value.trim();
  if (!outputPath) return setBackend('Choose an output path in Export settings');
  try {
    $('#exportError').textContent = '';
    exportJobId = await invoke('export_start', { project: clone(project), outputPath });
    $('#cancelExport').disabled = false;
    $('#exportButton').disabled = true;
    $('#exportButtonSide').disabled = true;
    setBackend(`Export ${exportJobId} started`);
    clearInterval(exportPoll);
    exportPoll = setInterval(pollExport, 400);
    pollExport();
  } catch (error) {
    $('#exportError').textContent = String(error);
    setBackend(`Export failed to start: ${error}`);
  }
}

async function pollExport() {
  if (!invoke || !exportJobId) return;
  try {
    const status = await invoke('export_status', { id: exportJobId });
    if (!status) return;
    const percent = Math.round(clamp(status.fraction || 0, 0, 1) * 100);
    $('#exportStage').textContent = status.stage;
    $('#exportPercent').textContent = `${percent}%`;
    $('#exportBar').style.width = `${percent}%`;
    $('#exportFrames').textContent = `${(status.completed_frames || 0).toLocaleString()} / ${(status.total_frames || 0).toLocaleString()}`;
    $('#exportError').textContent = status.error || '';
    if (status.done) {
      clearInterval(exportPoll);
      exportPoll = null;
      exportJobId = null;
      $('#cancelExport').disabled = true;
      $('#exportButton').disabled = false;
      $('#exportButtonSide').disabled = false;
      setBackend(status.cancelled ? 'Export cancelled' : status.error ? `Export failed: ${status.error}` : `Export complete: ${status.output_path}`);
    }
  } catch (error) {
    setBackend(`Export status error: ${error}`);
  }
}

async function cancelExport() {
  if (!invoke || !exportJobId) return;
  try {
    await invoke('export_cancel', { id: exportJobId });
    $('#cancelExport').disabled = true;
  } catch (error) {
    setBackend(`Cancel failed: ${error}`);
  }
}

function bindText(selector, getter, setter) {
  $(selector).addEventListener('input', () => {
    if (!project) return;
    setter($(selector).value, getter());
  });
}

function bindUi() {
  $$('.tab').forEach(tab => tab.addEventListener('click', () => {
    $$('.tab').forEach(item => item.classList.remove('active'));
    $$('.tab-page').forEach(page => page.classList.remove('active'));
    tab.classList.add('active');
    $(`[data-page="${tab.dataset.tab}"]`).classList.add('active');
  }));

  $('#projectName').addEventListener('input', () => mutateProject(p => { p.name = $('#projectName').value; }, true));
  $('#cardTitle').addEventListener('input', () => mutateCard(card => { card.title = $('#cardTitle').value; }));
  $('#cardDescription').addEventListener('input', () => mutateCard(card => { card.description = $('#cardDescription').value; }));
  $('#badgeValue').addEventListener('input', () => mutateCard(card => { card.badge_value = $('#badgeValue').value; }));
  $('#badgeLabel').addEventListener('input', () => mutateCard(card => { card.badge_label = $('#badgeLabel').value; }));
  $('#artworkPath').addEventListener('input', () => mutateCard(card => { card.artwork_path = $('#artworkPath').value.trim() || null; }));

  const updateArtworkColor = (value) => {
    const valid = /^#[0-9a-f]{6}$/i.test(value) ? value : '#000000';
    $('#artworkColor').value = valid;
    $('#artworkColorHex').value = valid.toUpperCase();
    mutateCard(card => { card.artwork_color = hexToRgba(valid, card.artwork_color?.[3] ?? 255); });
  };
  $('#artworkColor').addEventListener('input', () => updateArtworkColor($('#artworkColor').value));
  $('#artworkColorHex').addEventListener('change', () => updateArtworkColor($('#artworkColorHex').value));

  $('#modelKind').addEventListener('change', () => mutateProject(p => { p.model = $('#modelKind').value; }));
  $('#exportWidth').addEventListener('change', () => mutateProject(p => { p.export.width = Math.round(numberValue('#exportWidth', p.export.width)); }));
  $('#exportHeight').addEventListener('change', () => mutateProject(p => { p.export.height = Math.round(numberValue('#exportHeight', p.export.height)); }));
  $('#exportFps').addEventListener('change', () => mutateProject(p => { p.export.fps = Math.round(numberValue('#exportFps', p.export.fps)); }));
  $('#durationSeconds').addEventListener('change', () => mutateProject(p => { p.export.duration_seconds = numberValue('#durationSeconds', p.export.duration_seconds); }));
  $('#cardPitch').addEventListener('change', () => mutateProject(p => { p.theme.card_pitch_px = numberValue('#cardPitch', p.theme.card_pitch_px); }));
  $('#scrollSpeed').addEventListener('change', () => mutateProject(p => { p.theme.scroll_px_per_second = numberValue('#scrollSpeed', p.theme.scroll_px_per_second); }));
  $('#fontPath').addEventListener('change', () => mutateProject(p => { p.font_path = $('#fontPath').value.trim() || null; }));
  $('#soundtrackPath').addEventListener('change', () => mutateProject(p => { p.export.soundtrack_path = $('#soundtrackPath').value.trim() || null; }));
  $('#videoBitrate').addEventListener('change', () => mutateProject(p => { p.export.video_bitrate_mbps = Math.round(numberValue('#videoBitrate', p.export.video_bitrate_mbps)); }));
  $('#audioBitrate').addEventListener('change', () => mutateProject(p => { p.export.audio_bitrate_kbps = Math.round(numberValue('#audioBitrate', p.export.audio_bitrate_kbps)); }));
  $('#projectSavePath').addEventListener('input', () => { $('#projectSavePath').dataset.edited = '1'; });
  $('#outputPath').addEventListener('input', () => { $('#outputPath').dataset.edited = '1'; });

  $('#addCard').addEventListener('click', () => {
    const source = selected() || project.cards[project.cards.length - 1];
    const card = clone(source);
    card.id = `card-${Date.now()}`;
    card.title = `New card ${project.cards.length + 1}`;
    project.cards.splice(selectedCard + 1, 0, card);
    selectedCard += 1;
    setDirty(true); populateAll();
  });
  $('#duplicateCard').addEventListener('click', () => {
    const card = clone(selected());
    if (!card) return;
    card.id = `card-${Date.now()}`;
    project.cards.splice(selectedCard + 1, 0, card);
    selectedCard += 1;
    setDirty(true); populateAll();
  });
  $('#deleteCard').addEventListener('click', () => {
    if (project.cards.length <= 1) return setBackend('A project needs at least one card');
    project.cards.splice(selectedCard, 1);
    selectedCard = clamp(selectedCard, 0, project.cards.length - 1);
    setDirty(true); populateAll();
  });
  $('#moveCardUp').addEventListener('click', () => {
    if (selectedCard <= 0) return;
    [project.cards[selectedCard - 1], project.cards[selectedCard]] = [project.cards[selectedCard], project.cards[selectedCard - 1]];
    selectedCard -= 1; setDirty(true); populateAll();
  });
  $('#moveCardDown').addEventListener('click', () => {
    if (selectedCard >= project.cards.length - 1) return;
    [project.cards[selectedCard + 1], project.cards[selectedCard]] = [project.cards[selectedCard], project.cards[selectedCard + 1]];
    selectedCard += 1; setDirty(true); populateAll();
  });

  $('#frameSlider').addEventListener('input', () => setFrame(Number($('#frameSlider').value)));
  $('#playPause').addEventListener('click', () => playing ? stopPlayback() : startPlayback());
  $('#jumpStart').addEventListener('click', () => setFrame(0, true));
  $('#jumpEnd').addEventListener('click', () => setFrame(durationFrames() - 1, true));
  $('#sampleReference').addEventListener('click', sampleReferenceFrame);
  $('#referenceJump').addEventListener('click', async () => { setFrame(Math.min(3000, durationFrames() - 1), true); await sampleReferenceFrame(); });

  $('#newProject').addEventListener('click', createNewProject);
  $('#importProjectFile').addEventListener('change', (event) => {
    const file = event.target.files?.[0];
    if (file) importProjectFile(file);
    event.target.value = '';
  });
  $('#downloadProject').addEventListener('click', downloadProjectJson);
  $('#saveProject').addEventListener('click', saveProject);
  $('#exportButton').addEventListener('click', startExport);
  $('#exportButtonSide').addEventListener('click', startExport);
  $('#cancelExport').addEventListener('click', cancelExport);

  window.addEventListener('keydown', event => {
    if (event.code === 'Space' && !['INPUT','TEXTAREA','SELECT'].includes(document.activeElement?.tagName)) {
      event.preventDefault();
      playing ? stopPlayback() : startPlayback();
    }
    if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 's') {
      event.preventDefault();
      saveProject();
    }
  });
}

async function bootstrap() {
  bindUi();
  if (!invoke) {
    setBackend('Tauri bridge unavailable — run WatchCompare as the desktop/mobile app');
    return;
  }
  try {
    [referenceProfile, platform] = await Promise.all([
      invoke('reference_profile'),
      invoke('platform_info'),
    ]);
    $('#refPitch').textContent = `${referenceProfile.geometry.card_pitch_px} px`;
    $('#refFrames').textContent = `${referenceProfile.frame_count.toLocaleString()} frames`;
    $('#platformBadge').textContent = platform.os.toUpperCase();
    setBackend(`${platform.os} backend ready · ${platform.mp4_encoder} export`);
    await createNewProject();
  } catch (error) {
    setBackend(`Startup failed: ${error}`);
  }
}

bootstrap();
