<?php

namespace App\Http\Controllers;

use Illuminate\Http\Request;
use App\Services\OsdrService;

class OsdrController extends Controller
{
    protected $osdrService;

    public function __construct(OsdrService $osdrService)
    {
        $this->osdrService = $osdrService;
    }

    public function index(Request $request)
    {
        $limit = (int) $request->query('limit', 20);

        $dto = $this->osdrService->getOsdrList($limit);

        return view('osdr', $dto->toArray());
    }
}
