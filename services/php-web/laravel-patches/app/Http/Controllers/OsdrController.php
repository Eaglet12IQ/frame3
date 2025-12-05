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
        try {
            $osdrItems = $this->osdrService->getOsdrListPaginated(20);

            return view('osdr', [
                'osdr_items' => $osdrItems,
            ]);
        } catch (\Exception $e) {
            return $this->errorResponse($e->getMessage(), $e->getCode() ?: 500);
        }
    }
}
