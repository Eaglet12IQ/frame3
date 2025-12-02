<?php

namespace App\Http\Controllers;

use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use App\Support\JwstHelper;

class DashboardController extends Controller
{
    private function base(): string { return getenv('RUST_BASE') ?: 'http://rust_iss:3000'; }

    private function getJson(string $url, array $qs = []): array {
        if ($qs) $url .= (str_contains($url,'?')?'&':'?') . http_build_query($qs);
        $raw = @file_get_contents($url);
        return $raw ? (json_decode($raw, true) ?: []) : [];
    }

    public function index()
    {
        // минимум: карта МКС и пустые контейнеры, JWST-галерея подтянется через /api/jwst/feed
        $b     = $this->base();
        $iss   = $this->getJson($b.'/last');
        $trend = []; // фронт сам заберёт /api/iss/trend (через nginx прокси)

        // Fetch latest telemetry data from Pascal legacy service
        $telemetry = DB::select("SELECT recorded_at, voltage, temp, source_file FROM telemetry_legacy ORDER BY recorded_at DESC LIMIT 10");

        return view('dashboard', [
            'iss' => $iss,
            'trend' => $trend,
            'jw_gallery' => [], // не нужно сервером
            'jw_observation_raw' => [],
            'jw_observation_summary' => [],
            'jw_observation_images' => [],
            'jw_observation_files' => [],
            'metrics' => [
                'iss_speed' => $iss['payload']['velocity'] ?? null,
                'iss_alt'   => $iss['payload']['altitude'] ?? null,
                'neo_total' => 0,
            ],
            'telemetry' => $telemetry,
        ]);
    }


}
