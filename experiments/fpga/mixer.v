module mixer_core (
    input clk,
    input rst_n,
    input [7:0] pred_0, // Linear
    input [7:0] pred_1, // Simple
    input [7:0] pred_2, // Graph
    input [7:0] pred_3, // Spectral
    input [7:0] pred_4, // LzMatch
    input [7:0] pred_5, // Padding/Expansion
    input [15:0] weight_0, // Q1.15 fixed point
    input [15:0] weight_1,
    input [15:0] weight_2,
    input [15:0] weight_3,
    input [15:0] weight_4,
    input [15:0] weight_5,
    output reg [7:0] prediction
);

    // Internal wires for multiplication
    // 8-bit pred * 16-bit weight = 24-bit result
    wire [23:0] prod_0;
    wire [23:0] prod_1;
    wire [23:0] prod_2;
    wire [23:0] prod_3;
    wire [23:0] prod_4;
    wire [23:0] prod_5;

    // Use DSP blocks for multiplication
    assign prod_0 = pred_0 * weight_0;
    assign prod_1 = pred_1 * weight_1;
    assign prod_2 = pred_2 * weight_2;
    assign prod_3 = pred_3 * weight_3;
    assign prod_4 = pred_4 * weight_4;
    assign prod_5 = pred_5 * weight_5;

    // Accumulator (Sum of products)
    reg [26:0] accumulator; // Need bits to prevent overflow

    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            accumulator <= 0;
            prediction <= 0;
        end else begin
            // Pipeline Stage 1: Sum
            accumulator <= prod_0 + prod_1 + prod_2 + prod_3 + prod_4 + prod_5;
            
            // Pipeline Stage 2: Normalize (Divide by 2^15 approx or assume weights sum to 1.0)
            // If weights sum to 1.0 (32768 in Q1.15), then dividing by 32768 is just taking top bits.
            // Result is in accumulator[26:0]. We want 8 bits.
            // Q1.15 * integer = Q9.15 (sort of).
            // We strip the fractional 15 bits.
            
            // Simplistic rounding: add 0.5 (1 << 14) before truncation
            prediction <= (accumulator + 16384) >> 15;
        end
    end

endmodule

// ============================================================================
// TESTBENCH MODULE
// ============================================================================
module mixer_tb;
    reg clk;
    reg rst_n;
    reg [7:0] pred_0, pred_1, pred_2, pred_3, pred_4, pred_5;
    reg [15:0] weight_0, weight_1, weight_2, weight_3, weight_4, weight_5;
    wire [7:0] prediction;

    // Instantiate DUT
    mixer_core dut (
        .clk(clk),
        .rst_n(rst_n),
        .pred_0(pred_0), .pred_1(pred_1), .pred_2(pred_2),
        .pred_3(pred_3), .pred_4(pred_4), .pred_5(pred_5),
        .weight_0(weight_0), .weight_1(weight_1), .weight_2(weight_2),
        .weight_3(weight_3), .weight_4(weight_4), .weight_5(weight_5),
        .prediction(prediction)
    );

    // Clock generation (100 MHz)
    initial begin
        clk = 0;
        forever #5 clk = ~clk;
    end

    // Test sequence
    initial begin
        $dumpfile("mixer_tb.vcd");
        $dumpvars(0, mixer_tb);

        // Reset
        rst_n = 0;
        #20 rst_n = 1;

        // Test Case 1: Equal weights, predictions = 128
        weight_0 = 16'd5461; // ~1/6 in Q1.15
        weight_1 = 16'd5461;
        weight_2 = 16'd5461;
        weight_3 = 16'd5461;
        weight_4 = 16'd5461;
        weight_5 = 16'd5461;
        pred_0 = 8'd128; pred_1 = 8'd128; pred_2 = 8'd128;
        pred_3 = 8'd128; pred_4 = 8'd128; pred_5 = 8'd128;
        #20;
        $display("Test 1: prediction=%d (expected ~128)", prediction);

        // Test Case 2: Weighted toward pred_0
        weight_0 = 16'd32768; // 1.0 in Q1.15
        weight_1 = 0; weight_2 = 0; weight_3 = 0; weight_4 = 0; weight_5 = 0;
        pred_0 = 8'd200;
        #20;
        $display("Test 2: prediction=%d (expected ~200)", prediction);

        #50 $finish;
    end
endmodule
