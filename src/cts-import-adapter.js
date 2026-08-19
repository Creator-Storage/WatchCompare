const browserTextImport = importDataFile;

$('#dataFileInput').accept = '.csv,.tsv,.txt,.xlsx,text/csv,text/plain,application/vnd.openxmlformats-officedocument.spreadsheetml.sheet';

importDataFile = async function ctsImportDataFile(file) {
  if (!invoke) return browserTextImport(file);
  const lower = file.name.toLowerCase();
  if (!['.csv', '.tsv', '.txt', '.xlsx'].some((suffix) => lower.endsWith(suffix))) {
    throw new Error('Choose CSV, TSV, TXT, or XLSX data.');
  }
  const bytes = Array.from(new Uint8Array(await file.arrayBuffer()));
  const table = await invoke('import_table', { filename: file.name, bytes });
  const rows = [table.headers || [], ...(table.rows || [])];
  const cards = cardsFromRows(rows);
  state.project.cards = cards;
  state.selectedCardId = cards[0]?.id || null;
  const delimiter = '\t';
  $('#dataPaste').value = rows.map((row) => row.map((value) => {
    const text = String(value ?? '');
    return text.includes(delimiter) || text.includes('\n') || text.includes('"')
      ? `"${text.replaceAll('"', '""')}"`
      : text;
  }).join(delimiter)).join('\n');
  refreshDataDetection();
  persistProject();
  renderCards();
  renderPreview();
  showToast(`Imported ${cards.length} cards from ${file.name}.`);
};
