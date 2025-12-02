<?php

namespace App\Services;

use App\Clients\JwstClient;

class JwstService
{
    protected $jwstClient;

    public function __construct(JwstClient $jwstClient)
    {
        $this->jwstClient = $jwstClient;
    }

    public function getFeaturedObservation(): ?array
    {
        $selectedObs = session('jwst_selected_observation');

        if ($selectedObs && is_array($selectedObs)) {
            return $selectedObs;
        }

        $resp = $this->jwstClient->get('all/type/jpg', ['page' => 1, 'perPage' => 1]);
        $list = $resp['body'] ?? ($resp['data'] ?? (is_array($resp) ? $resp : []));

        if (!empty($list) && is_array($list)) {
            $item = $list[0];
            if (is_array($item)) {
                $url = $this->pickImageUrl($item);
                if ($url) {
                    $instList = $this->extractInstruments($item);

                    return [
                        'url' => $url,
                        'obs_id' => (string)($item['observation_id'] ?? $item['observationId'] ?? ''),
                        'program' => (string)($item['program'] ?? ''),
                        'suffix' => (string)($item['details']['suffix'] ?? $item['suffix'] ?? ''),
                        'instruments' => $instList,
                        'title' => trim($item['title'] ?? ''),
                        'description' => trim($item['description'] ?? ''),
                        'link' => $item['location'] ?? $url,
                        'details' => $item['details'] ?? [],
                    ];
                }
            }
        }

        return null;
    }

    public function selectObservation(array $data): bool
    {
        $obsId = $data['obs_id'] ?? null;
        $program = $data['program'] ?? '';
        $suffix = $data['suffix'] ?? '';
        $inst = $data['inst'] ?? [];
        $url = $data['url'] ?? null;
        $link = $data['link'] ?? null;

        if ($obsId && $url) {
            $featured = [
                'url' => $url,
                'obs_id' => $obsId,
                'program' => $program,
                'suffix' => $suffix,
                'instruments' => is_array($inst) ? $inst : [],
                'title' => '',
                'description' => '',
                'link' => $link ?: $url,
                'details' => [],
            ];

            session(['jwst_selected_observation' => $featured]);
            return true;
        }

        return false;
    }

    public function getFeed(array $params = []): array
    {
        $src = $params['source'] ?? 'jpg';
        $sfx = trim($params['suffix'] ?? '');
        $prog = trim($params['program'] ?? '');
        $instF = strtoupper(trim($params['instrument'] ?? ''));
        $page = max(1, (int)($params['page'] ?? 1));
        $per = max(1, min(60, (int)($params['perPage'] ?? 24)));

        $path = 'all/type/jpg';
        if ($src === 'suffix' && $sfx !== '') {
            $path = 'all/suffix/' . ltrim($sfx, '/');
        }
        if ($src === 'program' && $prog !== '') {
            $path = 'program/id/' . rawurlencode($prog);
        }

        $resp = $this->jwstClient->get($path, ['page' => $page, 'perPage' => $per]);
        $list = $resp['body'] ?? ($resp['data'] ?? (is_array($resp) ? $resp : []));

        $items = [];
        foreach ($list as $it) {
            if (!is_array($it)) continue;

            $url = $this->pickImageUrl($it);
            if (!$url) continue;

            $instList = $this->extractInstruments($it);
            if ($instF && $instList && !in_array($instF, $instList, true)) continue;

            $items[] = [
                'url' => $url,
                'obs' => (string)($it['observation_id'] ?? $it['observationId'] ?? ''),
                'program' => (string)($it['program'] ?? ''),
                'suffix' => (string)($it['details']['suffix'] ?? $it['suffix'] ?? ''),
                'inst' => $instList,
                'caption' => trim(
                    (($it['observation_id'] ?? '') ?: ($it['id'] ?? '')) .
                    ' · P' . ($it['program'] ?? '-') .
                    (($it['details']['suffix'] ?? '') ? ' · ' . $it['details']['suffix'] : '') .
                    ($instList ? ' · ' . implode('/', $instList) : '')
                ),
                'link' => $it['location'] ?? $url,
            ];
            if (count($items) >= $per) break;
        }

        return [
            'source' => $path,
            'count' => count($items),
            'items' => $items,
        ];
    }

    private function pickImageUrl(array $item): ?string
    {
        $loc = $item['location'] ?? $item['url'] ?? null;
        $thumb = $item['thumbnail'] ?? null;
        foreach ([$loc, $thumb] as $u) {
            if (is_string($u) && preg_match('~\.(jpg|jpeg|png)(\?.*)?$~i', $u)) {
                return $u;
            }
        }
        return $this->jwstClient->pickImageUrl($item);
    }

    private function extractInstruments(array $item): array
    {
        $instList = [];
        foreach (($item['details']['instruments'] ?? []) as $I) {
            if (is_array($I) && !empty($I['instrument'])) {
                $instList[] = strtoupper($I['instrument']);
            }
        }
        return $instList;
    }
}
