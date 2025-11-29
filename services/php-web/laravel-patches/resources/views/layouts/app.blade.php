<!doctype html>
<html lang="ru">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Space Dashboard</title>
  <link href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.3/dist/css/bootstrap.min.css" rel="stylesheet">
  <link rel="stylesheet" href="https://unpkg.com/leaflet@1.9.4/dist/leaflet.css"/>
  <style>
    #map{height:340px}
    /* Frontend Enhancements */
    .fade-in { animation: fadeIn 0.5s ease-in; }
    .slide-in { animation: slideIn 0.5s ease-out; }
    .card { transition: transform 0.2s, box-shadow 0.2s; }
    .card:hover { transform: translateY(-2px); box-shadow: 0 4px 8px rgba(0,0,0,0.1); }
    .table-responsive { animation: fadeIn 0.8s ease-in; }
    .btn { transition: all 0.2s; }
    .btn:hover { transform: scale(1.05); }
    @keyframes fadeIn { from { opacity: 0; } to { opacity: 1; } }
    @keyframes slideIn { from { transform: translateX(-20px); opacity: 0; } to { transform: translateX(0); opacity: 1; } }
    .sortable { cursor: pointer; user-select: none; }
    .sortable:hover { background-color: #f8f9fa; }
    .sortable.asc::after { content: ' ↑'; }
    .sortable.desc::after { content: ' ↓'; }
  </style>
  <script src="https://unpkg.com/leaflet@1.9.4/dist/leaflet.js"></script>
  <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
</head>
<body>
<nav class="navbar navbar-expand-lg bg-body-tertiary mb-3">
  <div class="container">
    <a class="navbar-brand" href="/dashboard">Dashboard</a>
    <a class="nav-link" href="/iss">ISS</a>
    <a class="nav-link" href="/jwst">JWST</a>
    <a class="nav-link" href="/osdr">OSDR</a>
    <a class="nav-link" href="/astro">Astro</a>
  </div>
</nav>
@yield('content')
<script src="https://cdn.jsdelivr.net/npm/bootstrap@5.3.3/dist/js/bootstrap.bundle.min.js"></script>
</body>
</html>
