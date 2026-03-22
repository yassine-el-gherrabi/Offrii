import SwiftUI

// MARK: - WishlistPreviewSection

struct WishlistPreviewSection: View {
    let items: [Item]

    var body: some View {
        VStack(alignment: .leading, spacing: OffriiTheme.spacingSM) {
            // Section title + "See all"
            HStack {
                Text(NSLocalizedString("home.wishlistPreview.title", comment: ""))
                    .font(OffriiTypography.headline)
                    .foregroundColor(OffriiTheme.text)

                Spacer()

                HStack(spacing: OffriiTheme.spacingXXS) {
                    Text(NSLocalizedString("home.wishlistPreview.seeAll", comment: ""))
                        .font(OffriiTypography.subheadline)
                    Image(systemName: "arrow.right")
                        .font(.system(size: 12))
                }
                .foregroundColor(OffriiTheme.primary)
            }

            // Item rows
            VStack(spacing: 0) {
                ForEach(Array(items.prefix(3).enumerated()), id: \.element.id) { index, item in
                    compactItemRow(item)

                    if index < min(items.count, 3) - 1 {
                        Divider()
                            .padding(.leading, OffriiTheme.spacingBase)
                    }
                }
            }
            .padding(OffriiTheme.spacingBase)
            .background(OffriiTheme.card)
            .cornerRadius(OffriiTheme.cornerRadiusLG)
            .shadow(
                color: OffriiTheme.cardShadowColor,
                radius: OffriiTheme.cardShadowRadius,
                x: 0,
                y: OffriiTheme.cardShadowY
            )
        }
    }

    // MARK: - Compact Item Row

    private func compactItemRow(_ item: Item) -> some View {
        HStack(spacing: OffriiTheme.spacingSM) {
            RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusXS)
                .fill(OffriiTheme.primary.opacity(0.1))
                .frame(width: 36, height: 36)
                .overlay(
                    Image(systemName: "gift.fill")
                        .foregroundColor(OffriiTheme.primary)
                        .font(.system(size: 14))
                )

            VStack(alignment: .leading, spacing: 2) {
                Text(item.name)
                    .font(OffriiTypography.subheadline)
                    .foregroundColor(OffriiTheme.text)
                    .lineLimit(1)

                HStack(spacing: -1) {
                    ForEach(0..<item.priority, id: \.self) { _ in
                        Image(systemName: "flame.fill")
                            .font(.system(size: 10))
                    }
                }
                .foregroundColor(priorityColor(for: item.priority))
            }

            Spacer()

            if let price = item.estimatedPrice {
                Text(formatPrice(price))
                    .font(OffriiTypography.subheadline)
                    .fontWeight(.semibold)
                    .foregroundColor(OffriiTheme.text)
            }

            if item.isClaimed {
                Image(systemName: "checkmark.circle.fill")
                    .font(.system(size: 14))
                    .foregroundColor(OffriiTheme.accent)
            }
        }
        .padding(.vertical, OffriiTheme.spacingXS)
    }

    private func priorityColor(for priority: Int) -> Color {
        switch priority {
        case 1: return OffriiTheme.primary.opacity(0.4)
        case 3: return OffriiTheme.primary
        default: return OffriiTheme.primary.opacity(0.7)
        }
    }

    private func formatPrice(_ price: Decimal) -> String {
        let formatter = NumberFormatter()
        formatter.numberStyle = .currency
        formatter.currencyCode = "EUR"
        return formatter.string(from: price as NSDecimalNumber) ?? "\(price) \u{20AC}"
    }
}
