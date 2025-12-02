<?php

namespace App\Services;

use App\Clients\RustClient;
use App\DTOs\OsdrListDto;

class OsdrService
{
    private RustClient $client;

    public function __construct(RustClient $client)
    {
        $this->client = $client;
    }

    public function getOsdrList(int $limit = 20): OsdrListDto
    {
        $data = $this->client->getJson('osdr/list', ['limit' => $limit]);
        $items = $data['items'] ?? [];

        $items = $this->flattenOsdr($items);

        return new OsdrListDto($items, $this->client->getBaseUrl() . '/osdr/list?limit=' . $limit);
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
