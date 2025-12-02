<?php

namespace App\Http\Controllers;

use Illuminate\Http\Request;
use App\Services\IssService;

class IssController extends Controller
{
    protected $issService;

    public function __construct(IssService $issService)
    {
        $this->issService = $issService;
    }

    public function index()
    {
        $last = $this->issService->getLast();
        $trend = $this->issService->getTrend(240);
        $base = getenv('RUST_BASE') ?: 'http://rust_iss:3000';

        return view('iss', [
            'last' => $last,
            'trend' => $trend,
            'base' => $base,
        ]);
    }

    public function last()
    {
        $data = $this->issService->getLast();
        return response()->json($data);
    }

    public function trend(Request $request)
    {
        $limit = $request->query('limit', 240);
        $data = $this->issService->getTrend($limit);
        return response()->json($data);
    }
}
