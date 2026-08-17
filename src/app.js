const sampleOutput = document.querySelector('#sampleOutput');
const sampleButton = document.querySelector('#sampleButton');

async function loadProfile() {
  try {
    const invoke = window.__TAURI__?.core?.invoke;
    if (!invoke) return;
    const profile = await invoke('reference_profile');
    document.querySelector('#pitch').textContent = `${profile.geometry.card_pitch_px} px`;
    document.querySelector('#speed').textContent = `${profile.motion.steady_scroll_px_per_second.toFixed(3)} px/s`;
    document.querySelector('#frames').textContent = profile.frame_count.toLocaleString();
  } catch (error) {
    console.warn('reference_profile unavailable', error);
  }
}

sampleButton?.addEventListener('click', async () => {
  try {
    const invoke = window.__TAURI__?.core?.invoke;
    if (!invoke) throw new Error('Tauri bridge unavailable');
    const state = await invoke('sample_reference', { frame: 3000 });
    sampleOutput.textContent = `frame ${state.frame} · ${state.time_seconds.toFixed(3)}s · ${state.stage} · x ${state.card_train_x_px.toFixed(3)} px · phase ${state.card_phase_px.toFixed(3)} px`;
  } catch (error) {
    sampleOutput.textContent = String(error);
  }
});

for (const button of document.querySelectorAll('.rail-button')) {
  button.addEventListener('click', () => {
    document.querySelectorAll('.rail-button').forEach(item => item.classList.remove('active'));
    button.classList.add('active');
  });
}

loadProfile();
