@extends('layouts.app')

@section('content')
<div class="container pb-5">
  {{-- ASTRO — события --}}
  <div class="col-12 order-first mt-3">
    <div class="card shadow-sm">
      <div class="card-body">
        <div class="d-flex justify-content-between align-items-center mb-2">
          <h5 class="card-title m-0 fade-in">Астрономические события (AstronomyAPI)</h5>
          <form id="astroForm" class="row g-2 align-items-center">
            <div class="col-auto">
              <input type="number" step="0.0001" class="form-control form-control-sm" name="lat" value="55.7558" placeholder="lat">
            </div>
            <div class="col-auto">
              <input type="number" step="0.0001" class="form-control form-control-sm" name="lon" value="37.6176" placeholder="lon">
            </div>
            <div class="col-auto">
              <input type="number" step="0.1" class="form-control form-control-sm" name="elevation" value="7" style="width:90px" title="высота (м)">
            </div>
            <div class="col-auto">
              <button class="btn btn-sm btn-primary" type="submit">Показать</button>
            </div>
          </form>
        </div>

        <div class="mb-3">
          <input type="text" id="astroSearch" class="form-control" placeholder="Поиск по ключевым словам...">
        </div>
        <div class="table-responsive">
          <table class="table table-sm align-middle">
            <thead>
              <tr>
                <th class="sortable" data-column="0">#</th>
                <th class="sortable" data-column="1">Тело</th>
                <th class="sortable" data-column="2">Событие</th>
                <th class="sortable" data-column="3">Когда (UTC)</th>
                <th>Дополнительно</th>
              </tr>
            </thead>
            <tbody id="astroBody">
              <tr><td colspan="5" class="text-muted">нет данных</td></tr>
            </tbody>
          </table>
        </div>

        <details class="mt-2">
          <summary>Полный JSON</summary>
          <pre id="astroRaw" class="bg-light rounded p-2 small m-0" style="white-space:pre-wrap"></pre>
        </details>
      </div>
    </div>
  </div>
</div>

<script>
document.addEventListener('DOMContentLoaded', () => {
  const form = document.getElementById('astroForm');
  const body = document.getElementById('astroBody');
  const raw  = document.getElementById('astroRaw');
  const searchInput = document.getElementById('astroSearch');
  let originalRows = [];

  function normalize(node){
    const name = node.name || node.body || node.object || node.target || '';
    const type = node.type || node.event_type || node.category || node.kind || '';
    const when = node.time || node.date || node.occursAt || node.peak || node.instant || '';
    const extra = node.magnitude || node.mag || node.altitude || node.note || '';
    return {name, type, when, extra};
  }

  function collect(root){
    const rows = [];
    if (root && root.data && root.data.rows) {
      root.data.rows.forEach(row => {
        if (row.body && row.events) {
          row.events.forEach(event => {
            rows.push({
              name: row.body.name || row.body.id || '—',
              type: event.type || '—',
              when: event.eventHighlights ? (event.eventHighlights.peak ? event.eventHighlights.peak.date : (event.eventHighlights.partialStart ? event.eventHighlights.partialStart.date : '—')) : '—',
              extra: event.extraInfo ? (event.extraInfo.obscuration ? `Obscuration: ${event.extraInfo.obscuration}` : '') : ''
            });
          });
        }
      });
    }
    return rows;
  }

  async function load(q){
    body.innerHTML = '<tr><td colspan="5" class="text-muted">Загрузка…</td></tr>';
    const url = '/api/astro/events?' + new URLSearchParams(q).toString();
    try{
      const r  = await fetch(url);
      const js = await r.json();
      raw.textContent = JSON.stringify(js, null, 2);

      const rows = collect(js);
      if (!rows.length) {
        body.innerHTML = '<tr><td colspan="5" class="text-muted">события не найдены</td></tr>';
        originalRows = [];
        return;
      }
      body.innerHTML = rows.slice(0,200).map((r,i)=>`
        <tr>
          <td>${i+1}</td>
          <td>${r.name || '—'}</td>
          <td>${r.type || '—'}</td>
          <td><code>${r.when || '—'}</code></td>
          <td>${r.extra || ''}</td>
        </tr>
      `).join('');

      // Update originalRows after loading new data
      originalRows = Array.from(body.querySelectorAll('tr')).filter(row => !row.classList.contains('collapse'));

      // Reinitialize table functionality after data load
      reinitTable();
    }catch(e){
      body.innerHTML = '<tr><td colspan="5" class="text-danger">ошибка загрузки</td></tr>';
      originalRows = [];
    }
  }

  form.addEventListener('submit', ev=>{
    ev.preventDefault();
    const q = Object.fromEntries(new FormData(form).entries());
    load(q);
  });

  // автозагрузка
  load({lat: form.lat.value, lon: form.lon.value, days: form.days.value});

  // Sorting and filtering functionality
  let sortDirection = {};

  function initTable() {
    originalRows = Array.from(body.querySelectorAll('tr')).filter(row => !row.classList.contains('collapse'));
    addSortListeners();
    addSearchListener();
  }

  function reinitTable() {
    // Remove existing listeners
    document.querySelectorAll('.sortable').forEach(header => {
      header.removeEventListener('click', header._sortHandler);
    });
    if (searchInput._searchHandler) {
      searchInput.removeEventListener('input', searchInput._searchHandler);
    }

    // Reinitialize
    initTable();
  }

  function addSortListeners() {
    document.querySelectorAll('.sortable').forEach(header => {
      header.addEventListener('click', () => {
        const column = parseInt(header.dataset.column);
        sortDirection[column] = !sortDirection[column];
        sortTable(column, sortDirection[column]);
        updateSortIndicators();
      });
    });
  }

  function addSearchListener() {
    searchInput.addEventListener('input', filterTable);
  }

  function sortTable(column, ascending) {
    const rows = Array.from(originalRows);

    rows.sort((a, b) => {
      const aVal = a.cells[column].textContent.trim().toLowerCase();
      const bVal = b.cells[column].textContent.trim().toLowerCase();

      if (ascending) {
        return aVal.localeCompare(bVal);
      } else {
        return bVal.localeCompare(aVal);
      }
    });

    body.innerHTML = '';
    rows.forEach(row => body.appendChild(row));
  }

  function filterTable() {
    console.log('Search triggered:', searchInput.value);
    const searchTerm = searchInput.value.toLowerCase();
    const allRows = body.querySelectorAll('tr');

    allRows.forEach(row => {
      const text = row.textContent.toLowerCase();
      if (text.includes(searchTerm)) {
        row.style.display = '';
      } else {
        row.style.display = 'none';
      }
    });
  }

  function updateSortIndicators() {
    document.querySelectorAll('.sortable').forEach(header => {
      header.classList.remove('asc', 'desc');
    });

    Object.keys(sortDirection).forEach(column => {
      const header = document.querySelector(`.sortable[data-column="${column}"]`);
      if (header) {
        header.classList.add(sortDirection[column] ? 'asc' : 'desc');
      }
    });
  }

  // Initialize after load
  setTimeout(initTable, 100);
});
</script>
@endsection
