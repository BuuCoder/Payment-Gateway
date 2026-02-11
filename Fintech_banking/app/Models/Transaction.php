<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;
use Illuminate\Database\Eloquent\Relations\HasOne;

class Transaction extends Model
{
    protected $fillable = [
        'from_account_id',
        'to_account_id',
        'amount',
        'currency',
        'type',
        'status',
        'description',
        'reference_number',
        'metadata',
    ];

    protected $casts = [
        'amount' => 'decimal:2',
        'metadata' => 'array',
    ];

    /**
     * Get the from account
     */
    public function fromAccount(): BelongsTo
    {
        return $this->belongsTo(Account::class, 'from_account_id');
    }

    /**
     * Get the to account
     */
    public function toAccount(): BelongsTo
    {
        return $this->belongsTo(Account::class, 'to_account_id');
    }

    /**
     * Get the payment for this transaction
     */
    public function payment(): HasOne
    {
        return $this->hasOne(Payment::class);
    }

    /**
     * Generate unique reference number
     */
    public static function generateReferenceNumber(): string
    {
        do {
            $number = 'TXN' . date('Ymd') . strtoupper(substr(uniqid(), -8));
        } while (self::where('reference_number', $number)->exists());

        return $number;
    }
}

