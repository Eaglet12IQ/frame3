<?php

namespace App\Services;

class IssService
{
    private function base(): string
    {
        return getenv('RUST_BASE') ?: 'http://rust_iss:3000';
    }

    private function getJson(string $url, array $qs = []): array
    {
        if ($qs) {
            $url .= (str_contains($url, '?') ? '&' : '?') . http_build_query($qs);
        }
        $raw = @file_get_contents($url);
        return $raw ? (json_decode($raw, true) ?: []) : [];
    }

    public function getLast(): array
    {
        return $this->getJson($this->base() . '/last');
    }

    public function getTrend(int $limit = 240): array
    {
        return $this->getJson($this->base() . '/iss/trend', ['limit' => $limit]);
    }
}
