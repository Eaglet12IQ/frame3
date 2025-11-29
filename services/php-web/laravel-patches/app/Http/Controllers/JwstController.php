<?php

namespace App\Http\Controllers;

use Illuminate\Http\Request;
use App\Support\JwstHelper;

class JwstController extends Controller
{
    public function index(Request $request)
    {
        $jw = new JwstHelper();
        $featured = null;

        // Проверяем, сохранено ли выбранное наблюдение в сессии
        $selectedObs = session('jwst_selected_observation');

        if ($selectedObs && is_array($selectedObs)) {
            // Используем сохраненное наблюдение
            $featured = $selectedObs;
        } else {
            // Показываем последнее наблюдение по умолчанию
            $resp = $jw->get('all/type/jpg', ['page' => 1, 'perPage' => 1]);
            $list = $resp['body'] ?? ($resp['data'] ?? (is_array($resp) ? $resp : []));

            if (!empty($list) && is_array($list)) {
                $item = $list[0];
                if (is_array($item)) {
                    // выбираем валидную картинку
                    $url = null;
                    $loc = $item['location'] ?? $item['url'] ?? null;
                    $thumb = $item['thumbnail'] ?? null;
                    foreach ([$loc, $thumb] as $u) {
                        if (is_string($u) && preg_match('~\.(jpg|jpeg|png)(\?.*)?$~i', $u)) { $url = $u; break; }
                    }
                    if (!$url) {
                        $url = \App\Support\JwstHelper::pickImageUrl($item);
                    }

                    if ($url) {
                        // фильтр по инструменту
                        $instList = [];
                        foreach (($item['details']['instruments'] ?? []) as $I) {
                            if (is_array($I) && !empty($I['instrument'])) $instList[] = strtoupper($I['instrument']);
                        }

                        $featured = [
                            'url' => $url,
                            'obs_id' => (string)($item['observation_id'] ?? $item['observationId'] ?? ''),
                            'program' => (string)($item['program'] ?? ''),
                            'suffix' => (string)($item['details']['suffix'] ?? $item['suffix'] ?? ''),
                            'instruments' => $instList,
                            'title' => trim($item['title'] ?? ''),
                            'description' => trim($item['description'] ?? ''),
                            'link' => $loc ?: $url,
                            'details' => $item['details'] ?? [],
                        ];
                    }
                }
            }
        }

        return view('jwst', [
            'featured' => $featured,
            'gallery' => [],
        ]);
    }

    /**
     * Выбор наблюдения из галереи
     */
    public function selectObservation(Request $request)
    {
        $obsId = $request->input('obs_id');
        $program = $request->input('program');
        $suffix = $request->input('suffix');
        $inst = $request->input('inst', []);
        $url = $request->input('url');
        $link = $request->input('link');
        $caption = $request->input('caption');

        if ($obsId && $url) {
            $featured = [
                'url' => $url,
                'obs_id' => $obsId,
                'program' => $program ?: '',
                'suffix' => $suffix ?: '',
                'instruments' => is_array($inst) ? $inst : [],
                'title' => '',
                'description' => '',
                'link' => $link ?: $url,
                'details' => [],
            ];

            // Сохраняем в сессии
            session(['jwst_selected_observation' => $featured]);

            return response()->json(['success' => true]);
        }

        return response()->json(['success' => false], 400);
    }

    /**
     * /api/jwst/feed — серверный прокси/нормализатор JWST картинок.
     * QS:
     *  - source: jpg|suffix|program (default jpg)
     *  - suffix: напр. _cal, _thumb, _crf
     *  - program: ID программы (число)
     *  - instrument: NIRCam|MIRI|NIRISS|NIRSpec|FGS
     *  - page, perPage
     */
    public function feed(Request $r)
    {
        $src   = $r->query('source', 'jpg');
        $sfx   = trim((string)$r->query('suffix', ''));
        $prog  = trim((string)$r->query('program', ''));
        $instF = strtoupper(trim((string)$r->query('instrument', '')));
        $page  = max(1, (int)$r->query('page', 1));
        $per   = max(1, min(60, (int)$r->query('perPage', 24)));

        $jw = new JwstHelper();

        // выбираем эндпоинт
        $path = 'all/type/jpg';
        if ($src === 'suffix' && $sfx !== '') $path = 'all/suffix/'.ltrim($sfx,'/');
        if ($src === 'program' && $prog !== '') $path = 'program/id/'.rawurlencode($prog);

        $resp = $jw->get($path, ['page'=>$page, 'perPage'=>$per]);
        $list = $resp['body'] ?? ($resp['data'] ?? (is_array($resp) ? $resp : []));

        $items = [];
        foreach ($list as $it) {
            if (!is_array($it)) continue;

            // выбираем валидную картинку
            $url = null;
            $loc = $it['location'] ?? $it['url'] ?? null;
            $thumb = $it['thumbnail'] ?? null;
            foreach ([$loc, $thumb] as $u) {
                if (is_string($u) && preg_match('~\.(jpg|jpeg|png)(\?.*)?$~i', $u)) { $url = $u; break; }
            }
            if (!$url) {
                $url = \App\Support\JwstHelper::pickImageUrl($it);
            }
            if (!$url) continue;

            // фильтр по инструменту
            $instList = [];
            foreach (($it['details']['instruments'] ?? []) as $I) {
                if (is_array($I) && !empty($I['instrument'])) $instList[] = strtoupper($I['instrument']);
            }
            if ($instF && $instList && !in_array($instF, $instList, true)) continue;

            $items[] = [
                'url'      => $url,
                'obs'      => (string)($it['observation_id'] ?? $it['observationId'] ?? ''),
                'program'  => (string)($it['program'] ?? ''),
                'suffix'   => (string)($it['details']['suffix'] ?? $it['suffix'] ?? ''),
                'inst'     => $instList,
                'caption'  => trim(
                    (($it['observation_id'] ?? '') ?: ($it['id'] ?? '')) .
                    ' · P' . ($it['program'] ?? '-') .
                    (($it['details']['suffix'] ?? '') ? ' · ' . $it['details']['suffix'] : '') .
                    ($instList ? ' · ' . implode('/', $instList) : '')
                ),
                'link'     => $loc ?: $url,
            ];
            if (count($items) >= $per) break;
        }

        return response()->json([
            'source' => $path,
            'count'  => count($items),
            'items'  => $items,
        ]);
    }
}
