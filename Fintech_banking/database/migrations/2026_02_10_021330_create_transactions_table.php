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
        Schema::create('transactions', function (Blueprint $table) {
            $table->id();
            $table->foreignId('from_account_id')->nullable()->constrained('accounts')->onDelete('set null');
            $table->foreignId('to_account_id')->nullable()->constrained('accounts')->onDelete('set null');
            $table->decimal('amount', 15, 2);
            $table->string('currency', 3)->default('VND');
            $table->enum('type', ['TRANSFER', 'DEPOSIT', 'WITHDRAWAL', 'PAYMENT'])->default('TRANSFER');
            $table->enum('status', ['PENDING', 'PROCESSING', 'SUCCESS', 'FAILED', 'CANCELLED'])->default('PENDING');
            $table->text('description')->nullable();
            $table->string('reference_number', 50)->unique();
            $table->json('metadata')->nullable();
            $table->timestamps();
            
            $table->index('from_account_id');
            $table->index('to_account_id');
            $table->index('reference_number');
            $table->index('status');
            $table->index('created_at');
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::dropIfExists('transactions');
    }
};
