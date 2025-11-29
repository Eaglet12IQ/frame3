@extends('layouts.app')

@section('content')
<div class="container py-4">
  <h3 class="mb-3">Астрономические события</h3>

  <div class="row g-3">
    <div class="col-md-12">
      <div class="card shadow-sm">
        <div class="card-body">
          <h5 class="card-title">События</h5>
          <div id="astro-events" class="text-muted">Загрузка...</div>
          <div class="mt-3">
            <button class="btn btn-primary" onclick="loadAstroEvents()">Загрузить события</button>
          </div>
        </div>
      </div>
    </div>
  </div>
</div>

<script>
function loadAstroEvents() {
  fetch('/api/astro/events')
    .then(response => response.json())
    .then(data => {
      const container = document.getElementById('astro-events');
      if (data.error) {
        container.innerHTML = '<div class="alert alert-danger">' + data.error + '</div>';
        return;
      }
      let html = '<ul class="list-group">';
      (data.events || []).forEach(event => {
        html += '<li class="list-group-item">' + (event.title || 'Unknown') + ' - ' + (event.date || '') + '</li>';
      });
      html += '</ul>';
      container.innerHTML = html;
    })
    .catch(error => {
      document.getElementById('astro-events').innerHTML = '<div class="alert alert-danger">Ошибка загрузки</div>';
    });
}
</script>
@endsection
