<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    /**
     * Run the migrations.
     */
    public function up(): void
    {
        Schema::create('payments', function (Blueprint $table) {
            $table->id();
            $table->foreignId('transaction_id')->nullable()->constrained()->onDelete('set null');
            $table->foreignId('user_id')->constrained()->onDelete('cascade');
            $table->decimal('amount', 15, 2);
            $table->string('currency', 3)->default('VND');
            $table->enum('payment_method', ['CARD', 'BANK_TRANSFER', 'EWALLET', 'CASH'])->default('CARD');
            $table->string('merchant_id', 100)->nullable();
            $table->enum('status', ['PENDING', 'PROCESSING', 'SUCCESS', 'FAILED', 'RETRY', 'FRAUD_DETECTED', 'CANCELLED'])->default('PENDING');
            $table->string('error_code', 50)->nullable();
            $table->text('error_message')->nullable();
            $table->integer('retry_count')->default(0);
            $table->json('metadata')->nullable();
            $table->timestamp('processed_at')->nullable();
            $table->timestamps();
            
            $table->index('transaction_id');
            $table->index('user_id');
            $table->index('merchant_id');
            $table->index('status');
            $table->index('created_at');
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::dropIfExists('payments');
    }
};
