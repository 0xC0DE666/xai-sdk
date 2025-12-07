use anyhow::{Context, Result};
use std::env;
use xai_sdk::Request;
use xai_sdk::api::management::billing::{
    GetAmountToPayReq, GetBillingInfoReq, GetSpendingLimitsReq, ListPaymentMethodsReq,
};
use xai_sdk::billing;

#[tokio::main]
async fn main() -> Result<()> {
    println!("💳 xAI Billing Service Example");
    println!("==============================\n");

    // Load API key for authentication
    let api_key =
        env::var("XAI_API_KEY").context("XAI_API_KEY environment variable must be set")?;

    // Create authenticated billing client
    let mut client = billing::client::new(&api_key).await?;

    // Get team ID from environment (or use the one from your API key info)
    let team_id = env::var("XAI_TEAM_ID").unwrap_or_else(|_| "your-team-id".to_string());

    println!("📋 Getting Billing Information");
    println!("-----------------------------\n");

    // Get billing info
    let billing_info_req = Request::new(GetBillingInfoReq {
        team_id: team_id.clone(),
    });

    match client.get_billing_info(billing_info_req).await {
        Ok(response) => {
            let billing_info = response.into_inner();
            if let Some(info) = billing_info.billing_info {
                println!("✅ Billing info retrieved successfully\n");
                println!("👤 Name: {}", info.name);
                println!("📧 Email: {}", info.email);
                if let Some(address) = info.address {
                    println!("📍 Address:");
                    println!("   Line 1: {}", address.line1);
                    if !address.line2.is_empty() {
                        println!("   Line 2: {}", address.line2);
                    }
                    println!("   City: {}", address.city);
                    println!("   State: {}", address.state);
                    println!("   Postal Code: {}", address.postal_code);
                    println!("   Country: {}", address.country);
                }
                if !info.tax_id_type.is_empty() {
                    println!("🏛️  Tax ID Type: {}", info.tax_id_type);
                }
                if !info.tax_number.is_empty() {
                    println!("🔢 Tax Number: {}", info.tax_number);
                }
            } else {
                println!("ℹ️  No billing information set for this team");
            }
        }
        Err(e) => {
            eprintln!("❌ Error fetching billing info: {}", e);
        }
    }

    println!("\n💳 Listing Payment Methods");
    println!("---------------------------\n");

    // List payment methods
    let payment_methods_req = Request::new(ListPaymentMethodsReq {
        team_id: team_id.clone(),
    });

    match client.list_payment_methods(payment_methods_req).await {
        Ok(response) => {
            let methods = response.into_inner();
            println!("✅ Payment methods retrieved successfully\n");

            if methods.payment_methods.is_empty() {
                println!("ℹ️  No payment methods on file");
            } else {
                println!(
                    "📝 Found {} payment method(s):\n",
                    methods.payment_methods.len()
                );
                for (idx, method) in methods.payment_methods.iter().enumerate() {
                    println!(
                        "  {}. Payment Method ID: {}",
                        idx + 1,
                        method.payment_method_id
                    );
                    println!("     Type: {}", method.payment_type);

                    if let Some(card) = &method.card_details {
                        println!("     Card: {} ending in {}", card.brand, card.last4);
                        println!("     Expires: {}/{}", card.exp_month, card.exp_year);
                    }

                    if let Some(ach) = &method.us_bank_account_details {
                        println!("     Bank: {} ending in {}", ach.bank_name, ach.last4);
                        println!("     Routing: {}", ach.routing_number);
                    }

                    if let Some(link) = &method.link_details {
                        println!("     Link: {}", link.email);
                    }

                    println!();
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Error listing payment methods: {}", e);
        }
    }

    println!("💰 Getting Amount to Pay");
    println!("------------------------\n");

    // Get amount to pay for current billing period
    let amount_req = Request::new(GetAmountToPayReq {
        team_id: team_id.clone(),
    });

    match client.get_amount_to_pay(amount_req).await {
        Ok(response) => {
            let amount_info = response.into_inner();
            println!("✅ Amount to pay retrieved successfully\n");

            if let Some(cycle) = amount_info.billing_cycle {
                println!("📅 Billing Cycle: {}/{}", cycle.month, cycle.year);
            }

            println!(
                "💵 Effective Spending Limit: ${:.2}",
                amount_info.effective_spending_limit as f64 / 100.0
            );
            println!(
                "🎁 Default Credits: ${:.2}",
                amount_info.default_credits as f64 / 100.0
            );

            if let Some(invoice) = amount_info.core_invoice {
                println!("\n📄 Current Invoice:");
                println!(
                    "   Amount Before VAT: ${:.2}",
                    invoice.amount_before_vat as f64 / 100.0
                );
                println!("   VAT: ${:.2}", invoice.vat_cost as f64 / 100.0);
                println!(
                    "   Amount After VAT: ${:.2}",
                    invoice.amount_after_vat as f64 / 100.0
                );

                if let Some(total) = invoice.total_with_corr {
                    println!("   Total: ${:.2}", total.val as f64 / 100.0);
                }

                if let Some(prepaid) = invoice.prepaid_credits {
                    println!(
                        "   Prepaid Credits Available: ${:.2}",
                        prepaid.val as f64 / 100.0
                    );
                }

                if let Some(used) = invoice.prepaid_credits_used {
                    println!("   Prepaid Credits Used: ${:.2}", used.val as f64 / 100.0);
                }

                if !invoice.lines.is_empty() {
                    println!("\n   Line Items:");
                    for line in &invoice.lines {
                        println!(
                            "     - {}: {} {} @ ${:.6} = ${:.2}",
                            line.description,
                            line.num_units,
                            line.unit_type,
                            line.unit_price as f64 / 1_000_000.0,
                            line.amount as f64 / 100.0
                        );
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Error getting amount to pay: {}", e);
        }
    }

    println!("\n📊 Getting Spending Limits");
    println!("--------------------------\n");

    // Get spending limits
    let limits_req = Request::new(GetSpendingLimitsReq { team_id });

    match client.get_spending_limits(limits_req).await {
        Ok(response) => {
            let resp = response.into_inner();
            if let Some(limits) = resp.spending_limits {
                println!("✅ Spending limits retrieved successfully\n");

                if let Some(hard_auto) = limits.hard_sl_auto {
                    println!("💵 Hard Limit (Auto): ${:.2}", hard_auto.val as f64 / 100.0);
                }
                
                if let Some(effective_hard) = limits.effective_hard_sl {
                    println!("💵 Effective Hard Limit: ${:.2}", effective_hard.val as f64 / 100.0);
                }

                if let Some(soft) = limits.soft_sl {
                    println!("💵 Soft Limit (User Set): ${:.2}", soft.val as f64 / 100.0);
                } else {
                    println!("💵 Soft Limit: Not set");
                }

                if let Some(effective) = limits.effective_sl {
                    println!("💵 Effective Limit (Enforced): ${:.2}", effective.val as f64 / 100.0);
                }
            } else {
                println!("ℹ️  No spending limits information available");
            }
        }
        Err(e) => {
            eprintln!("❌ Error getting spending limits: {}", e);
        }
    }

    Ok(())
}
