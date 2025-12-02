<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;

class Telemetry extends Model
{
    use HasFactory;

    protected $table = 'telemetry_legacy';

    public $timestamps = false;

    protected $fillable = [
        'recorded_at',
        'voltage',
        'temp',
        'source_file',
    ];

    protected $casts = [
        'recorded_at' => 'datetime',
        'voltage' => 'decimal:2',
        'temp' => 'decimal:2',
    ];
}
