<?php

namespace App\Services;

class OsdrService
{
    private function base(): string
    {
        return getenv('RUST_BASE') ?: 'http://rust_iss:3000';
    }

    private function getJson(string $url): array
    {
        $raw = @file_get_contents($url);
        return $raw ? json_decode($raw, true) : ['items' => []];
    }

    public function getOsdrList(int $limit = 20): array
    {
        $base = $this->base();
        $data = $this->getJson($base . '/osdr/list?limit=' . $limit);
        $items = $data['items'] ?? [];

        $items = $this->flattenOsdr($items);

        return [
            'items' => $items,
            'src' => $base . '/osdr/list?limit=' . $limit,
        ];
    }

    /** Преобразует данные вида {"OSD-1": {...}, "OSD-2": {...}} в плоский список */
    private function flattenOsdr(array $items): array
    {
        $out = [];
        foreach ($items as $row) {
            $raw = $row['raw'] ?? [];
            if (is_array($raw) && $this->looksOsdrDict($raw)) {
                foreach ($raw as $k => $v) {
                    if (!is_array($v)) continue;
                    $rest = $v['REST_URL'] ?? $v['rest_url'] ?? $v['rest'] ?? null;
                    $title = $v['title'] ?? $v['name'] ?? null;
                    if (!$title && is_string($rest)) {
                        // запасной вариант: последний сегмент URL как подпись
                        $title = basename(rtrim($rest, '/'));
                    }
                    $out[] = [
                        'id' => $row['id'],
                        'dataset_id' => $k,
                        'title' => $title,
                        'status' => $row['status'] ?? null,
                        'updated_at' => $row['updated_at'] ?? null,
                        'inserted_at' => $row['inserted_at'] ?? null,
                        'rest_url' => $rest,
                        'raw' => $v,
                    ];
                }
            } else {
                // обычная строка — просто прокинем REST_URL если найдётся
                $row['rest_url'] = is_array($raw) ? ($raw['REST_URL'] ?? $raw['rest_url'] ?? null) : null;
                $out[] = $row;
            }
        }
        return $out;
    }

    private function looksOsdrDict(array $raw): bool
    {
        // словарь ключей "OSD-xxx" ИЛИ значения содержат REST_URL
        foreach ($raw as $k => $v) {
            if (is_string($k) && str_starts_with($k, 'OSD-')) return true;
            if (is_array($v) && (isset($v['REST_URL']) || isset($v['rest_url']))) return true;
        }
        return false;
    }
}
