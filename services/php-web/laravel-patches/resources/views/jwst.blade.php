@extends('layouts.app')

@section('content')
<div class="container pb-5">
  <div class="row g-3">
<div class="container pb-5">
  <div class="row g-3">
    {{-- левая колонка: JWST наблюдение (как раньше было под APOD можно держать своим блоком) --}}
    <div class="col-lg-7">
      <div class="card shadow-sm h-100">
        <div class="card-body">
          <h5 class="card-title">Выбранное наблюдение</h5>
          @if($featured)
            <div class="row">
              <div class="col-md-6">
                <a href="{{ $featured['link'] }}" target="_blank" rel="noreferrer">
                  <img src="{{ $featured['url'] }}" class="img-fluid rounded mb-2" alt="JWST Featured">
                </a>
              </div>
              <div class="col-md-6">
                <dl class="row small">
                  <dt class="col-sm-4">ID:</dt>
                  <dd class="col-sm-8">{{ $featured['obs_id'] ?: '—' }}</dd>

                  <dt class="col-sm-4">Программа:</dt>
                  <dd class="col-sm-8">{{ $featured['program'] ?: '—' }}</dd>

                  <dt class="col-sm-4">Инструмент:</dt>
                  <dd class="col-sm-8">{{ $featured['instruments'] ? implode('/', $featured['instruments']) : '—' }}</dd>

                  <dt class="col-sm-4">Суффикс:</dt>
                  <dd class="col-sm-8">{{ $featured['suffix'] ?: '—' }}</dd>
                </dl>
                @if($featured['title'])
                  <p class="mt-2 mb-1"><strong>{{ $featured['title'] }}</strong></p>
                @endif
                @if($featured['description'])
                  <p class="small text-muted">{{ $featured['description'] }}</p>
                @endif
              </div>
            </div>
            <details class="mt-2">
              <summary>Полный JSON</summary>
              <pre class="bg-light rounded p-2 small m-0" style="white-space:pre-wrap">{{ json_encode($featured['details'], JSON_PRETTY_PRINT | JSON_UNESCAPED_UNICODE) }}</pre>
            </details>
          @else
            <div class="text-muted">Наблюдение не найдено</div>
          @endif
        </div>
      </div>
    </div>

    {{-- НИЖНЯЯ ПОЛОСА: НОВАЯ ГАЛЕРЕЯ JWST --}}
    <div class="col-12">
      <div class="card shadow-sm">
        <div class="card-body">
          <div class="d-flex justify-content-between align-items-center mb-2">
            <h5 class="card-title m-0">Последние изображения</h5>
            <form id="jwstFilter" class="row g-2 align-items-center">
              <div class="col-auto">
                <select class="form-select form-select-sm" name="source" id="srcSel">
                  <option value="jpg" selected>Все JPG</option>
                  <option value="suffix">По суффиксу</option>
                  <option value="program">По программе</option>
                </select>
              </div>
              <div class="col-auto">
                <input type="text" class="form-control form-control-sm" name="suffix" id="suffixInp" placeholder="_cal / _thumb" style="width:140px;display:none">
                <input type="text" class="form-control form-control-sm" name="program" id="progInp" placeholder="2734" style="width:110px;display:none">
              </div>
              <div class="col-auto">
                <select class="form-select form-select-sm" name="instrument" style="width:130px">
                  <option value="">Любой инструмент</option>
                  <option>NIRCam</option><option>MIRI</option><option>NIRISS</option><option>NIRSpec</option><option>FGS</option>
                </select>
              </div>
              <div class="col-auto">
                <select class="form-select form-select-sm" name="perPage" style="width:90px">
                  <option>12</option><option selected>24</option><option>36</option><option>48</option>
                </select>
              </div>
              <div class="col-auto">
                <button class="btn btn-sm btn-primary" type="submit">Показать</button>
              </div>
            </form>
          </div>

          <style>
            .jwst-slider{position:relative}
            .jwst-track{
              display:flex; gap:.75rem; overflow:auto; scroll-snap-type:x mandatory; padding:.25rem;
            }
            .jwst-item{flex:0 0 180px; scroll-snap-align:start}
            .jwst-item img{width:100%; height:180px; object-fit:cover; border-radius:.5rem}
            .jwst-cap{font-size:.85rem; margin-top:.25rem}
            .jwst-nav{position:absolute; top:40%; transform:translateY(-50%); z-index:2}
            .jwst-prev{left:-.25rem} .jwst-next{right:-.25rem}
          </style>

          <div class="jwst-slider">
            <button class="btn btn-light border jwst-nav jwst-prev" type="button" aria-label="Prev">‹</button>
            <div id="jwstTrack" class="jwst-track border rounded"></div>
            <button class="btn btn-light border jwst-nav jwst-next" type="button" aria-label="Next">›</button>
          </div>

          <div id="jwstInfo" class="small text-muted mt-2"></div>
        </div>
      </div>
    </div>
  </div>
</div>

<script>
document.addEventListener('DOMContentLoaded', async function () {
  // ====== ПРОВЕРКА ВЫБРАННОГО НАБЛЮДЕНИЯ В localStorage ======
  const featuredObs = localStorage.getItem('jwst_featured_obs');
  if (featuredObs) {
    try {
      const obsData = JSON.parse(featuredObs);
      updateFeaturedObservation(obsData);
    } catch (e) {
      console.error('Error parsing featured observation:', e);
      updateFeaturedObservation(null);
    }
  } else {
    updateFeaturedObservation(null);
  }

  const urlParams = new URLSearchParams(window.location.search);
  if (urlParams.get('selected') === '1') {
    // Очищаем URL параметр после загрузки
    window.history.replaceState({}, document.title, '/jwst');
  }

  // Функция для обновления блока выбранного наблюдения
  function updateFeaturedObservation(obsData) {
    const featuredCard = document.querySelector('.card-body h5');
    if (!featuredCard) return;

    const cardBody = featuredCard.parentElement;

    let content;
    if (obsData) {
      // Создаем новый контент с данными наблюдения
      content = `
        <h5 class="card-title">Выбранное наблюдение</h5>
        <div class="row">
          <div class="col-md-6">
            <a href="${obsData.link}" target="_blank" rel="noreferrer">
              <img src="${obsData.url}" class="img-fluid rounded mb-2" alt="JWST Featured">
            </a>
          </div>
          <div class="col-md-6">
            <dl class="row small">
              <dt class="col-sm-4">ID:</dt>
              <dd class="col-sm-8">${obsData.obs_id || '—'}</dd>
              <dt class="col-sm-4">Программа:</dt>
              <dd class="col-sm-8">${obsData.program || '—'}</dd>
              <dt class="col-sm-4">Инструмент:</dt>
              <dd class="col-sm-8">${obsData.instruments ? obsData.instruments.join('/') : '—'}</dd>
              <dt class="col-sm-4">Суффикс:</dt>
              <dd class="col-sm-8">${obsData.suffix || '—'}</dd>
            </dl>
          </div>
        </div>
      `;
    } else {
      // Заглушка, если нет выбранного наблюдения
      content = `
        <h5 class="card-title">Выбранное наблюдение</h5>
        <div class="d-flex align-items-center justify-content-center p-4 border border-dashed rounded bg-light">
          <div class="text-center text-muted">
            <i class="bi bi-image fs-1 mb-2"></i>
            <p class="mb-0">Нужно выбрать наблюдение из галереи ниже</p>
          </div>
        </div>
      `;
    }

    cardBody.innerHTML = content;
  }

  // ====== JWST ГАЛЕРЕЯ ======
  const track = document.getElementById('jwstTrack');
  const info  = document.getElementById('jwstInfo');
  const form  = document.getElementById('jwstFilter');
  const srcSel = document.getElementById('srcSel');
  const sfxInp = document.getElementById('suffixInp');
  const progInp= document.getElementById('progInp');

  function toggleInputs(){
    sfxInp.style.display  = (srcSel.value==='suffix')  ? '' : 'none';
    progInp.style.display = (srcSel.value==='program') ? '' : 'none';
  }
  srcSel.addEventListener('change', toggleInputs); toggleInputs();

  async function loadFeed(qs){
    track.innerHTML = '<div class="p-3 text-muted">Загрузка…</div>';
    info.textContent= '';
    try{
      const url = '/api/jwst/feed?'+new URLSearchParams(qs).toString();
      const r = await fetch(url);
      const js = await r.json();
      track.innerHTML = '';
      (js.items||[]).forEach(it=>{
        const fig = document.createElement('figure');
        fig.className = 'jwst-item m-0';
        fig.innerHTML = `
          <div class="jwst-image-container" data-obs-id="${it.obs}" style="cursor:pointer;">
            <img loading="lazy" src="${it.url}" alt="JWST">
          </div>
          <figcaption class="jwst-cap">${(it.caption||'').replaceAll('<','<')}</figcaption>`;
        track.appendChild(fig);

        // Добавляем обработчик клика для выбора наблюдения
        const imgContainer = fig.querySelector('.jwst-image-container');
        imgContainer.addEventListener('click', function() {
          console.log('Image clicked:', it.obs);
          // Сохраняем в localStorage
          const obsData = {
            obs_id: it.obs,
            program: it.program,
            suffix: it.suffix,
            instruments: it.inst,
            url: it.url,
            link: it.link,
            caption: it.caption
          };
          console.log('Saving to localStorage:', obsData);
          localStorage.setItem('jwst_featured_obs', JSON.stringify(obsData));

          // Обновляем блок выбранного наблюдения без перезагрузки страницы
          updateFeaturedObservation(obsData);
        });
      });
      info.textContent = `Источник: ${js.source} · Показано ${js.count||0}`;
    }catch(e){
      track.innerHTML = '<div class="p-3 text-danger">Ошибка загрузки</div>';
    }
  }

  form.addEventListener('submit', function(ev){
    ev.preventDefault();
    const fd = new FormData(form);
    const q = Object.fromEntries(fd.entries());
    loadFeed(q);
  });

  // навигация
  document.querySelector('.jwst-prev').addEventListener('click', ()=> track.scrollBy({left:-600, behavior:'smooth'}));
  document.querySelector('.jwst-next').addEventListener('click', ()=> track.scrollBy({left: 600, behavior:'smooth'}));

  // стартовые данные
  loadFeed({source:'jpg', perPage:24});
});
</script>
@endsection
