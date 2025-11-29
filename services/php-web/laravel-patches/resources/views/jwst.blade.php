@extends('layouts.app')

@section('content')
<div class="container py-4">
  <h3 class="mb-3">JWST Галерея</h3>

  <div class="row g-3">
    <div class="col-md-12">
      <div class="card shadow-sm">
        <div class="card-body">
          <h5 class="card-title">Галерея изображений</h5>
          <div id="jwst-gallery" class="row">
            <!-- Images will be loaded here -->
          </div>
          <div class="mt-3">
            <button class="btn btn-primary" onclick="loadJwstGallery()">Загрузить галерею</button>
          </div>
        </div>
      </div>
    </div>
  </div>
</div>

<script>
function loadJwstGallery() {
  fetch('/api/jwst/feed')
    .then(response => response.json())
    .then(data => {
      const container = document.getElementById('jwst-gallery');
      container.innerHTML = '';
      (data.items || []).forEach(item => {
        const col = document.createElement('div');
        col.className = 'col-md-4 mb-3';
        col.innerHTML = `
          <div class="card">
            <img src="${item.url}" class="card-img-top" alt="${item.caption}">
            <div class="card-body">
              <p class="card-text">${item.caption}</p>
            </div>
          </div>
        `;
        container.appendChild(col);
      });
    })
    .catch(error => {
      document.getElementById('jwst-gallery').innerHTML = '<div class="alert alert-danger">Ошибка загрузки галереи</div>';
    });
}
</script>
@endsection
