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
        $data = $request->only(['obs_id', 'program', 'suffix', 'inst', 'url', 'link', 'caption']);

        $success = $this->jwstService->selectObservation($data);

        return response()->json(['success' => $success], $success ? 200 : 400);
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
        $params = $r->query();

        $dto = $this->jwstService->getFeed($params);

        return response()->json($dto->toArray());
    }
}
