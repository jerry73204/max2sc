//! Basic example of using the max2sc testing framework

use max2sc_test::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing for logging
    tracing_subscriber::fmt::init();

    // Create a test runner (requires SuperCollider to be installed)
    let runner = match SCTestRunner::new() {
        Ok(runner) => runner,
        Err(e) => {
            println!("⚠️  SuperCollider not found: {e}");
            println!(
                "📋 To run this example, install SuperCollider and ensure 'sclang' is in your PATH"
            );
            return Ok(());
        }
    };

    println!("🎵 max2sc Testing Framework Example");
    println!("=====================================");

    // Example 1: Syntax test
    println!("\n1️⃣  Testing SuperCollider syntax validation...");
    let syntax_test = SyntaxTest::new("SinOsc.ar(440, 0, 0.5)");
    let result = runner
        .run_test(TestCategory::Syntax, async {
            syntax_test.run(&runner).await
        })
        .await?;

    if result.data.is_success() {
        println!("✅ Syntax test passed in {:?}", result.duration);
    } else {
        println!("❌ Syntax test failed:");
        for error in &result.data.errors {
            println!("   Error: {}", error.message);
        }
    }

    // Example 2: Functional test
    println!("\n2️⃣  Testing object instantiation...");
    let functional_test =
        FunctionalTest::new("~osc = SinOsc.ar(440)").assert(Assertion::not_nil("~osc"));

    let result = runner
        .run_test(TestCategory::Functional, async {
            functional_test.run(&runner).await
        })
        .await?;

    println!("✅ Functional test completed in {:?}", result.duration);
    println!("   Pass rate: {:.1}%", result.data.pass_rate());

    // Example 3: Test fixtures
    println!("\n3️⃣  Testing with fixtures...");
    let fixture = TestData::simple_sine(440.0);
    println!("📄 Loaded fixture: {}", fixture.name);
    println!("   Description: {}", fixture.metadata.description);
    println!("   Category: {:?}", fixture.metadata.category);
    println!("   Boxes: {}", fixture.patch.patcher.boxes.len());

    // Example 4: Show available test fixtures
    println!("\n4️⃣  Available test fixtures:");
    let fixtures = TestData::standard_fixtures();
    for (i, fixture) in fixtures.iter().enumerate() {
        println!(
            "   {}. {} - {}",
            i + 1,
            fixture.name,
            fixture.metadata.description
        );
    }

    println!("\n🎉 Testing framework example completed!");
    println!(
        "💡 This framework enables comprehensive validation of Max to SuperCollider conversions"
    );

    Ok(())
}
