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
                    OffriiHaptics.selection()
                    selectedCategory = category
                } label: {
                    Text(category.label)
                        .font(OffriiTypography.footnote)
                        .fontWeight(.medium)
                        .frame(maxWidth: .infinity)
                        .padding(.vertical, OffriiTheme.spacingSM + 2)
                        .background(
                            selectedCategory == category
                                ? OffriiTheme.primary
                                : OffriiTheme.surface
                        )
                        .foregroundColor(
                            selectedCategory == category
                                ? .white
                                : OffriiTheme.textSecondary
                        )
                        .cornerRadius(OffriiTheme.cornerRadiusMD)
                        .overlay(
                            RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusMD)
                                .strokeBorder(
                                    selectedCategory == category
                                        ? OffriiTheme.primary
                                        : Color.clear,
                                    lineWidth: 1.5
                                )
                        )
                }
                .buttonStyle(.plain)
                .animation(OffriiAnimation.snappy, value: selectedCategory)
            }
        }
    }
}
