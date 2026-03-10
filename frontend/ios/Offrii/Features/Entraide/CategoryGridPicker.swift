import SwiftUI

// MARK: - CategoryGridPicker

struct CategoryGridPicker: View {
    @Binding var selectedCategory: WishCategory?

    private let columns = [
        GridItem(.flexible()),
        GridItem(.flexible()),
    ]

    var body: some View {
        LazyVGrid(columns: columns, spacing: OffriiTheme.spacingSM) {
            ForEach(WishCategory.allCases) { category in
                Button {
                    selectedCategory = category
                } label: {
                    HStack(spacing: OffriiTheme.spacingSM) {
                        Text(category.emoji)
                            .font(.system(size: 20))
                        Text(category.label)
                            .font(OffriiTypography.footnote)
                            .fontWeight(.medium)
                    }
                    .frame(maxWidth: .infinity)
                    .padding(.vertical, OffriiTheme.spacingSM + 2)
                    .background(
                        selectedCategory == category
                            ? category.textColor.opacity(0.12)
                            : OffriiTheme.cardSurface
                    )
                    .foregroundColor(
                        selectedCategory == category
                            ? category.textColor
                            : OffriiTheme.textSecondary
                    )
                    .cornerRadius(OffriiTheme.cornerRadiusMD)
                    .overlay(
                        RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusMD)
                            .strokeBorder(
                                selectedCategory == category
                                    ? category.textColor
                                    : Color.clear,
                                lineWidth: 1.5
                            )
                    )
                }
                .buttonStyle(.plain)
            }
        }
    }
}
