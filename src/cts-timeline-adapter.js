/* CTS owns project duration; WatchCompare owns the measured renderer motion. */
function ctsProjectFrameCount() {
  return Math.max(1, Math.round(projectDuration() * (state.profile.fps || 60)));
}

function ctsProjectStage(frame) {
  const fps = state.profile.fps || 60;
  const seconds = frame / fps;
  const duration = projectDuration();
  if (duration - seconds <= 0.8) return 'fade';
  const visible = Math.min(state.project.cards.length, 4);
  const intro = visible * 2;
  if (seconds < intro) return 'intro';
  if (seconds >= Math.max(intro, duration - 2.8)) return 'outro';
  return 'cruise';
}

const watchcompareSourceFrameState = getFrameState;
getFrameState = function ctsFrameState(frame) {
  const base = watchcompareSourceFrameState(Math.min(frame, state.profile.frame_count - 1));
  const pitch = state.profile.geometry.card_pitch_px || 477;
  const maxTravel = Math.max(0, state.project.cards.length - 4) * pitch;
  const measuredX = Number(base.card_train_x_px || 0);
  return {
    ...base,
    frame,
    time_seconds: frame / (state.profile.fps || 60),
    time_millis: frame / (state.profile.fps || 60) * 1000,
    stage: ctsProjectStage(frame),
    card_train_x_px: Math.max(-maxTravel, Math.min(0, measuredX)),
  };
};

outputTimeForFrame = function ctsOutputTimeForFrame(frame) {
  return Math.max(0, frame) / (state.profile.fps || 60);
};

setFrame = function ctsSetFrame(frame, { syncAudio: shouldSyncAudio = true } = {}) {
  const max = ctsProjectFrameCount() - 1;
  state.frame = Math.max(0, Math.min(max, Math.round(frame)));
  $('#timeline').max = String(max);
  $('#timeline').value = String(state.frame);
  const seconds = outputTimeForFrame(state.frame);
  $('#timeReadout').textContent = formatDuration(seconds, true);
  $('#frameReadout').textContent = `Frame ${state.frame.toLocaleString()}`;
  if (state.frame < state.profile.frame_count) requestScene(state.frame);
  else state.scene = null;
  renderPreview();
  if (shouldSyncAudio) syncAudio(false);
};

const watchcompareRenderPreview = renderPreview;
renderPreview = function ctsRenderPreview() {
  watchcompareRenderPreview();
  const remaining = projectDuration() - outputTimeForFrame(state.frame);
  if (remaining < 0.8) {
    const alpha = 1 - Math.max(0, remaining) / 0.8;
    ctx.save();
    ctx.fillStyle = `rgba(0,0,0,${Math.max(0, Math.min(1, alpha))})`;
    ctx.fillRect(0, 0, preview.width, preview.height);
    ctx.restore();
    homeCtx.clearRect(0, 0, homePreview.width, homePreview.height);
    homeCtx.drawImage(preview, 0, 0, homePreview.width, homePreview.height);
  }
};

play = function ctsPlay() {
  if (state.playing) return pause();
  const max = ctsProjectFrameCount() - 1;
  if (state.frame >= max) setFrame(0);
  state.playing = true;
  state.playStartFrame = state.frame;
  state.playStartWall = performance.now();
  $('#playButton').textContent = 'Ⅱ';
  syncAudio(true);
  const fps = state.profile.fps || 60;
  const tick = (now) => {
    if (!state.playing) return;
    const next = state.playStartFrame + (now - state.playStartWall) * fps / 1000;
    if (next >= max) {
      setFrame(max, { syncAudio: false });
      pause();
      return;
    }
    setFrame(next, { syncAudio: false });
    updateAudioVolumes(outputTimeForFrame(state.frame));
    state.raf = requestAnimationFrame(tick);
  };
  state.raf = requestAnimationFrame(tick);
};

const watchcompareUpdateDurationUi = updateDurationUi;
updateDurationUi = function ctsUpdateDurationUi() {
  watchcompareUpdateDurationUi();
  $('#timeline').max = String(ctsProjectFrameCount() - 1);
};

/* wireEvents captured the old play function by reference; replace only this button. */
const oldPlayButton = $('#playButton');
const newPlayButton = oldPlayButton.cloneNode(true);
oldPlayButton.replaceWith(newPlayButton);
newPlayButton.addEventListener('click', () => play());

/* loadRenderer is already running; whenever it reports back, restore the CTS range. */
const rendererStatusObserver = new MutationObserver(() => {
  updateDurationUi();
  setFrame(Math.min(state.frame, ctsProjectFrameCount() - 1), { syncAudio: false });
});
rendererStatusObserver.observe($('#rendererStatus'), { childList: true, characterData: true, subtree: true });

/* First launch should demonstrate the renderer, not look like an empty black preview. */
const initialVisibleFrame = Math.min(480, ctsProjectFrameCount() - 1);
setFrame(initialVisibleFrame);
