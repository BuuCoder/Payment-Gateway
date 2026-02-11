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
        Schema::create('accounts', function (Blueprint $table) {
            $table->id();
            $table->foreignId('user_id')->constrained()->onDelete('cascade');
            $table->string('account_number', 20)->unique();
            $table->enum('account_type', ['SAVINGS', 'CHECKING', 'BUSINESS'])->default('SAVINGS');
            $table->decimal('balance', 15, 2)->default(0);
            $table->string('currency', 3)->default('VND');
            $table->enum('status', ['ACTIVE', 'INACTIVE', 'SUSPENDED', 'CLOSED'])->default('ACTIVE');
            $table->timestamps();
            
            $table->index('user_id');
            $table->index('account_number');
            $table->index('status');
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::dropIfExists('accounts');
    }
};
