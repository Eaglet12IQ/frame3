<?php

namespace App\Http\Controllers;

use Illuminate\Http\Request;

class IssController extends Controller
{
    private function base(): string { return getenv('RUST_BASE') ?: 'http://rust_iss:3000'; }

    private function getJson(string $url, array $qs = []): array {
        if ($qs) $url .= (str_contains($url,'?')?'&':'?') . http_build_query($qs);
        $raw = @file_get_contents($url);
        return $raw ? (json_decode($raw, true) ?: []) : [];
    }

    public function index()
    {
        $b     = $this->base();
        $last  = $this->getJson($b.'/last');
        $trend = $this->getJson($b.'/iss/trend', ['limit' => 240]);

        return view('iss', [
            'last' => $last,
            'trend' => $trend,
            'base' => $b,
        ]);
    }
}
