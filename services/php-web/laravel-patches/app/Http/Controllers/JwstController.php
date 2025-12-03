<?php

namespace App\Http\Controllers;

use Illuminate\Http\Request;
use App\Services\JwstService;

class JwstController extends Controller
{
    protected $jwstService;

    public function __construct(JwstService $jwstService)
    {
        $this->jwstService = $jwstService;
    }

    public function index(Request $request)
    {
        $featured = $this->jwstService->getFeaturedObservation();

        return view('jwst', [
            'featured' => $featured,
            'gallery' => [],
        ]);
    }

    /**
     * Выбор наблюдения из галереи
     */
    public function selectObservation(Request $request)
    {
        try {
            $data = $request->validate([
                'obs_id' => 'required|string|max:255',
                'program' => 'nullable|string|max:255',
                'suffix' => 'nullable|string|max:255',
                'inst' => 'nullable|string|max:255',
                'url' => 'nullable|url',
                'link' => 'nullable|url',
                'caption' => 'nullable|string|max:1000',
            ]);

            $success = $this->jwstService->selectObservation($data);

            if ($success) {
                return response()->json(['success' => $success]);
            } else {
                return $this->errorResponse('Failed to select observation', 400);
            }
        } catch (\Exception $e) {
            return $this->errorResponse($e->getMessage(), $e->getCode() ?: 500);
        }
    }

    /**
     * /api/jwst/feed — серверный прокси/нормализатор JWST картинок.
     * QS:
     *  - source: jpg|suffix|program (default jpg)
     *  - suffix: напр. _cal, _thumb, _crf
     *  - program: ID программы (число)
     *  - instrument: NIRCam|MIRI|NIRISS|NIRSpec|FGS
     *  - page, perPage
     */
    public function feed(Request $r)
    {
        try {
            $validated = $r->validate([
                'source' => 'nullable|string|in:jpg,suffix,program',
                'suffix' => 'nullable|string|max:50',
                'program' => 'nullable|integer|min:1',
                'instrument' => 'nullable|string|in:NIRCam,MIRI,NIRISS,NIRSpec,FGS',
                'page' => 'nullable|integer|min:1',
                'perPage' => 'nullable|integer|min:1|max:100',
            ]);

            $dto = $this->jwstService->getFeed($validated);

            return response()->json($dto->toArray());
        } catch (\Exception $e) {
            return $this->errorResponse($e->getMessage(), $e->getCode() ?: 500);
        }
    }
}
