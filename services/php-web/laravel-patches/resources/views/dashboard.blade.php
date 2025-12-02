@extends('layouts.app')

@section('content')
<div class="container pb-5">
  <div class="row">
    <div class="col-12">
      <p class="text-muted fade-in">Добро пожаловать на Space Dashboard. Перейдите к отдельным разделам, чтобы получить подробную информацию.</p>
    </div>
  </div>

{{-- ===== CMS-блок из БД (нарочно сырая вставка) ===== --}}
<div class="card mt-3 slide-in">
  <div class="card-body">
    @php
      try {
        // «плохо»: запрос из Blade, без кэша, без репозитория
        $___b = DB::selectOne("SELECT content FROM cms_blocks WHERE slug='dashboard_experiment' AND is_active = TRUE LIMIT 1");
        echo $___b ? $___b->content : '<div class="text-muted">блок не найден</div>';
      } catch (\Throwable $e) {
        echo '<div class="text-danger">ошибка БД: '.e($e->getMessage()).'</div>';
      }
    @endphp
  </div>
</div>

{{-- ===== Таблица телеметрии из Pascal Legacy Service ===== --}}
<div class="card mt-3 shadow-sm slide-in">
  <div class="card-body">
    <div class="d-flex justify-content-between align-items-center mb-2">
      <h5 class="card-title m-0 fade-in">Телеметрия из Pascal Legacy Service</h5>
    </div>

    <div class="mb-3">
      <input type="text" id="telemetrySearch" class="form-control" placeholder="Поиск по данным...">
    </div>
    <div class="table-responsive">
      <table class="table table-sm align-middle">
        <thead>
          <tr>
            <th class="sortable" data-column="0">Время записи</th>
            <th class="sortable" data-column="1">Напряжение (V)</th>
            <th class="sortable" data-column="2">Температура (°C)</th>
            <th class="sortable" data-column="3">Источник</th>
          </tr>
        </thead>
        <tbody id="telemetryBody">
          @if(count($telemetry) > 0)
            @foreach($telemetry as $index => $record)
              <tr>
                <td>{{ \Carbon\Carbon::parse($record->recorded_at)->format('d.m.Y H:i:s') }}</td>
                <td>{{ number_format($record->voltage, 2) }}</td>
                <td>{{ number_format($record->temp, 2) }}</td>
                <td>{{ $record->source_file }}</td>
              </tr>
            @endforeach
          @else
            <tr><td colspan="4" class="text-muted">Нет данных телеметрии.</td></tr>
          @endif
        </tbody>
      </table>
    </div>
  </div>
</div>
</div>

<script>
document.addEventListener('DOMContentLoaded', () => {
  const body = document.getElementById('telemetryBody');
  const searchInput = document.getElementById('telemetrySearch');
  let originalRows = [];

  function initTable() {
    originalRows = Array.from(body.querySelectorAll('tr')).filter(row => !row.classList.contains('collapse'));
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

  // Sorting and filtering functionality
  let sortDirection = {};

  // Initialize after load
  setTimeout(initTable, 100);
});
</script>
@endsection
