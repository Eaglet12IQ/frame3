<?php

namespace App\Clients;

class AstroClient
{
    private string $appId;
    private string $secret;
    private string $auth;

    public function __construct()
    {
        $this->appId = env('ASTRO_APP_ID');
        $this->secret = env('ASTRO_APP_SECRET');

        if (!$this->appId || !$this->secret) {
            throw new \Exception('Missing ASTRO_APP_ID or ASTRO_APP_SECRET');
        }

        $this->auth = base64_encode($this->appId . ':' . $this->secret);
    }

    public function getEvents(array $params = []): array
    {
        $bodies = ['sun', 'moon'];
        $rows = [];

        foreach ($bodies as $body) {
            $url = 'https://api.astronomyapi.com/api/v2/bodies/events/' . urlencode($body) . '?' . http_build_query([
                'latitude' => (float)($params['lat'] ?? 65.9558),
                'longitude' => (float)($params['lon'] ?? 37.6171),
                'elevation' => (float)($params['elevation'] ?? 7),
                'from_date' => $params['from_date'] ?? '2025-11-29',
                'to_date' => $params['to_date'] ?? '2026-11-29',
                'time' => $params['time'] ?? '00:00:00',
                'output' => $params['output'] ?? 'rows',
            ]);

            $ch = curl_init($url);
            curl_setopt_array($ch, [
                CURLOPT_RETURNTRANSFER => true,
                CURLOPT_HTTPHEADER => [
                    'Authorization: Basic ' . $this->auth,
                    'Content-Type: application/json',
                ],
                CURLOPT_TIMEOUT => 20,
            ]);

            $raw = curl_exec($ch);
            $code = curl_getinfo($ch, CURLINFO_RESPONSE_CODE) ?: 0;
            $err = curl_error($ch);
            curl_close($ch);

            if ($raw === false || $code >= 400) {
                throw new \Exception($err ?: ("HTTP " . $code), $code ?: 500);
            }

            $data = json_decode($raw, true);
            if (isset($data['data']['rows'])) {
                $rows = array_merge($rows, $data['data']['rows']);
            }
        }

        return $rows;
    }
}
