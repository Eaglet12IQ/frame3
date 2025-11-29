@extends('layouts.app')

@section('content')
<div class="container py-4">
  <h3 class="mb-3">МКС данные</h3>

  <div class="row g-3">
    <div class="col-md-6">
      <div class="card shadow-sm">
        <div class="card-body">
          <h5 class="card-title">Последний снимок</h5>
          @if(!empty($last['payload']))
            <ul class="list-group">
              <li class="list-group-item">Широта {{ $last['payload']['latitude'] ?? '—' }}</li>
              <li class="list-group-item">Долгота {{ $last['payload']['longitude'] ?? '—' }}</li>
              <li class="list-group-item">Высота км {{ $last['payload']['altitude'] ?? '—' }}</li>
              <li class="list-group-item">Скорость км/ч {{ $last['payload']['velocity'] ?? '—' }}</li>
              <li class="list-group-item">Время {{ $last['fetched_at'] ?? '—' }}</li>
            </ul>
          @else
            <div class="text-muted">нет данных</div>
          @endif
        </div>
      </div>
    </div>

    <div class="col-md-6">
      <div class="card shadow-sm">
        <div class="card-body">
          <h5 class="card-title">Тренд движения</h5>
          @if(!empty($trend))
            <ul class="list-group">
              <li class="list-group-item">Движение {{ ($trend['movement'] ?? false) ? 'да' : 'нет' }}</li>
              <li class="list-group-item">Смещение км {{ number_format($trend['delta_km'] ?? 0, 3, '.', ' ') }}</li>
              <li class="list-group-item">Интервал сек {{ $trend['dt_sec'] ?? 0 }}</li>
              <li class="list-group-item">Скорость км/ч {{ $trend['velocity_kmh'] ?? '—' }}</li>
            </ul>
          @else
            <div class="text-muted">нет данных</div>
          @endif
        </div>
      </div>
    </div>
  </div>

  {{-- правая колонка: карта МКС --}}
  <div class="row g-3 mt-3">
    <div class="col-lg-8">
      <div class="card shadow-sm h-100">
        <div class="card-body">
          <h5 class="card-title">Положение и движение</h5>
          <div id="map" class="rounded mb-2 border" style="height:400px"></div>
          <div class="row g-2">
            <div class="col-6"><canvas id="issSpeedChart" height="110"></canvas></div>
            <div class="col-6"><canvas id="issAltChart"   height="110"></canvas></div>
          </div>
        </div>
      </div>
    </div>
  </div>
</div>

<script>
document.addEventListener('DOMContentLoaded', async function () {
  // ====== карта и графики МКС ======
  if (typeof L !== 'undefined' && typeof Chart !== 'undefined') {
    const last = @json(($last['payload'] ?? []));
    let lat0 = Number(last.latitude || 0), lon0 = Number(last.longitude || 0);
    const map = L.map('map', { attributionControl:false }).setView([lat0||0, lon0||0], lat0?3:2);
    L.tileLayer('https://{s}.tile.openstreetmap.de/{z}/{x}/{y}.png', { noWrap:true }).addTo(map);
    const trail  = L.polyline([], {weight:3}).addTo(map);
    const marker = L.marker([lat0||0, lon0||0]).addTo(map).bindPopup('МКС');

    const speedChart = new Chart(document.getElementById('issSpeedChart'), {
      type: 'line', data: { labels: [], datasets: [{ label: 'Скорость', data: [] }] },
      options: { responsive: true, scales: { x: { display: false } } }
    });
    const altChart = new Chart(document.getElementById('issAltChart'), {
      type: 'line', data: { labels: [], datasets: [{ label: 'Высота', data: [] }] },
      options: { responsive: true, scales: { x: { display: false } } }
    });

    async function loadTrend() {
      try {
        const r = await fetch('/api/iss/trend?limit=240');
        const js = await r.json();
        const pts = Array.isArray(js.points) ? js.points.map(p => [p.lat, p.lon]) : [];
        if (pts.length) {
          trail.setLatLngs(pts);
          marker.setLatLng(pts[pts.length-1]);
        }
        const t = (js.points||[]).map(p => new Date(p.at).toLocaleTimeString());
        speedChart.data.labels = t;
        speedChart.data.datasets[0].data = (js.points||[]).map(p => p.velocity);
        speedChart.update();
        altChart.data.labels = t;
        altChart.data.datasets[0].data = (js.points||[]).map(p => p.altitude);
        altChart.update();
      } catch(e) {}
    }
    loadTrend();
    setInterval(loadTrend, 15000);
  }
});
</script>
@endsection
