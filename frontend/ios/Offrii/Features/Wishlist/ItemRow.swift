import SwiftUI

struct ItemRow: View {
    let item: Item
    let categoryName: String?

    var body: some View {
        HStack(spacing: OffriiTheme.spacingBase) {
            // Gift icon
            RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusSM)
                .fill(OffriiTheme.primary.opacity(0.1))
                .frame(width: 44, height: 44)
                .overlay(
                    Image(systemName: "gift.fill")
                        .foregroundColor(OffriiTheme.primary)
                        .font(.system(size: 18))
                )

            // Name + category + priority
            VStack(alignment: .leading, spacing: OffriiTheme.spacingXS) {
                Text(item.name)
                    .font(OffriiTypography.headline)
                    .foregroundColor(OffriiTheme.text)
                    .lineLimit(1)

                HStack(spacing: OffriiTheme.spacingSM) {
                    if let categoryName {
                        Text(categoryName)
                            .font(OffriiTypography.caption)
                            .foregroundColor(OffriiTheme.textMuted)
                    }

                    Text(item.priorityLabel)
                        .font(OffriiTypography.caption)
                        .fontWeight(.medium)
                        .foregroundColor(priorityColor)
                }
            }

            Spacer()

            // Price + claimed badge
            VStack(alignment: .trailing, spacing: OffriiTheme.spacingXS) {
                if let price = item.estimatedPrice {
                    Text(formatPrice(price))
                        .font(OffriiTypography.subheadline)
                        .fontWeight(.semibold)
                        .foregroundColor(OffriiTheme.text)
                }

                if item.isClaimed {
                    Text(NSLocalizedString("wishlist.reserved", comment: ""))
                        .font(.system(size: 10, weight: .semibold))
                        .foregroundColor(OffriiTheme.accent)
                        .padding(.horizontal, 6)
                        .padding(.vertical, 2)
                        .background(OffriiTheme.accent.opacity(0.15))
                        .cornerRadius(OffriiTheme.cornerRadiusXS)
                }
            }
        }
        .padding(.vertical, OffriiTheme.spacingSM)
    }

    private var priorityColor: Color {
        switch item.priority {
        case 1: return OffriiTheme.textMuted
        case 3: return OffriiTheme.danger
        default: return OffriiTheme.accent
        }
    }

    private func formatPrice(_ price: Decimal) -> String {
        let formatter = NumberFormatter()
        formatter.numberStyle = .currency
        formatter.currencyCode = "EUR"
        return formatter.string(from: price as NSDecimalNumber) ?? "\(price) \u{20AC}"
    }
}
