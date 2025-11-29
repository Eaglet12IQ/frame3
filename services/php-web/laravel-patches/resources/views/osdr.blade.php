@extends('layouts.app')

@section('content')
<div class="container py-3">
  <h3 class="mb-3 fade-in">NASA OSDR</h3>

  <div class="mb-3">
    <input type="text" id="osdrSearch" class="form-control" placeholder="Поиск по ключевым словам...">
  </div>
  <div class="table-responsive">
    <table class="table table-sm table-striped align-middle">
      <thead>
        <tr>
          <th class="sortable" data-column="0">#</th>
          <th class="sortable" data-column="1">dataset_id</th>
          <th class="sortable" data-column="2">title</th>
          <th>REST_URL</th>
          <th class="sortable" data-column="4">updated_at</th>
          <th class="sortable" data-column="5">inserted_at</th>
          <th>raw</th>
        </tr>
      </thead>
      <tbody id="osdrBody">
      @forelse(array_slice($items, 0, 50) as $row)
        <tr class="fade-in">
          <td>{{ $row['id'] }}</td>
          <td>{{ $row['dataset_id'] ?? '—' }}</td>
          <td style="max-width:420px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap">
            {{ $row['title'] ?? '—' }}
          </td>
          <td>
            @if(!empty($row['rest_url']))
              <a href="{{ $row['rest_url'] }}" target="_blank" rel="noopener">открыть</a>
            @else — @endif
          </td>
          <td>{{ $row['updated_at'] ?? '—' }}</td>
          <td>{{ $row['inserted_at'] ?? '—' }}</td>
          <td>
            <button class="btn btn-outline-secondary btn-sm" data-bs-toggle="collapse" data-bs-target="#raw-{{ $row['id'] }}-{{ md5($row['dataset_id'] ?? (string)$row['id']) }}">JSON</button>
          </td>
        </tr>
        <tr class="collapse" id="raw-{{ $row['id'] }}-{{ md5($row['dataset_id'] ?? (string)$row['id']) }}">
          <td colspan="7">
            <pre class="mb-0" style="max-height:260px;overflow:auto">{{ json_encode($row['raw'] ?? [], JSON_PRETTY_PRINT|JSON_UNESCAPED_SLASHES|JSON_UNESCAPED_UNICODE) }}</pre>
          </td>
        </tr>
      @empty
        <tr><td colspan="7" class="text-center text-muted">нет данных</td></tr>
      @endforelse
      </tbody>
    </table>
  </div>
</div>

<script>
document.addEventListener('DOMContentLoaded', () => {
  // Sorting and filtering functionality
  let sortDirection = {};
  let originalRows = [];

  function initTable() {
    const tbody = document.getElementById('osdrBody');
    originalRows = Array.from(tbody.querySelectorAll('tr')).filter(row => !row.classList.contains('collapse'));
    addSortListeners();
    addSearchListener();
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
    const searchInput = document.getElementById('osdrSearch');
    searchInput.addEventListener('input', filterTable);
  }

  function sortTable(column, ascending) {
    const tbody = document.getElementById('osdrBody');
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

    tbody.innerHTML = '';
    rows.forEach(row => tbody.appendChild(row));
  }

  function filterTable() {
    const searchTerm = document.getElementById('osdrSearch').value.toLowerCase();

    originalRows.forEach(row => {
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
