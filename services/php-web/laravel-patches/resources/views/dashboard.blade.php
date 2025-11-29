@extends('layouts.app')

@section('content')
<div class="container pb-5">
  <div class="row">
    <div class="col-12">
      <p class="text-muted">Добро пожаловать на Space Dashboard. Перейдите к отдельным разделам, чтобы получить подробную информацию.</p>
    </div>
  </div>

{{-- ===== CMS-блок из БД (нарочно сырая вставка) ===== --}}
<div class="card mt-3">
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
</div>
@endsection
