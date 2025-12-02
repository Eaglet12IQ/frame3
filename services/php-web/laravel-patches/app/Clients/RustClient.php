<?php

namespace App\Clients;

class RustClient
{
    private string $baseUrl;

    public function __construct()
    {
        $this->baseUrl = getenv('RUST_BASE') ?: 'http://rust_iss:3000';
    }

    public function getJson(string $endpoint, array $queryParams = []): array
    {
        $url = $this->baseUrl . '/' . ltrim($endpoint, '/');
        if ($queryParams) {
            $url .= (str_contains($url, '?') ? '&' : '?') . http_build_query($queryParams);
        }

        $raw = @file_get_contents($url);
        return $raw ? (json_decode($raw, true) ?: []) : [];
    }

    public function getBaseUrl(): string
    {
        return $this->baseUrl;
    }
}
