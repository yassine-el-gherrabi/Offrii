import SwiftUI

// MARK: - CategoryChipItem Protocol

protocol CategoryChipItem: Identifiable, Equatable {
    var chipLabel: String { get }
    var chipIcon: String { get }
    var chipColor: Color { get }
}

// MARK: - CategoryChipsBar (reusable, single-select)

struct CategoryChipsBar<Item: CategoryChipItem>: View {
    let items: [Item]
    @Binding var selectedId: Item.ID?
    var allLabel: String = NSLocalizedString("wishlist.allCategories", comment: "")

    var body: some View {
        ScrollView(.horizontal, showsIndicators: false) {
            HStack(spacing: OffriiTheme.spacingSM) {
                let allSelected = selectedId == nil

                Button {
                    withAnimation(OffriiAnimation.snappy) {
                        selectedId = nil
                    }
                    OffriiHaptics.selection()
                } label: {
                    chipView(
                        icon: "sparkles",
                        label: allLabel,
                        isSelected: allSelected,
                        color: OffriiTheme.primary
                    )
                }
                .buttonStyle(.plain)

                ForEach(items) { item in
                    let isSelected = selectedId == item.id

                    Button {
                        withAnimation(OffriiAnimation.snappy) {
                            selectedId = isSelected ? nil : item.id
                        }
                        OffriiHaptics.selection()
                    } label: {
                        chipView(
                            icon: item.chipIcon,
                            label: item.chipLabel,
                            isSelected: isSelected,
                            color: item.chipColor
                        )
                    }
                    .buttonStyle(.plain)
                }
            }
            .padding(.horizontal, OffriiTheme.spacingBase)
            .padding(.vertical, OffriiTheme.spacingXS)
        }
    }

    private func chipView(icon: String, label: String, isSelected: Bool, color: Color) -> some View {
        HStack(spacing: 4) {
            Image(systemName: icon)
                .font(.system(size: 11))
            Text(label)
                .font(.system(size: 13, weight: isSelected ? .semibold : .regular))
        }
        .foregroundColor(isSelected ? .white : OffriiTheme.textSecondary)
        .padding(.horizontal, OffriiTheme.spacingMD)
        .padding(.vertical, OffriiTheme.spacingSM)
        .background(isSelected ? color : .white)
        .cornerRadius(OffriiTheme.cornerRadiusXL)
        .overlay(
            RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusXL)
                .strokeBorder(isSelected ? .clear : OffriiTheme.border, lineWidth: 1)
        )
    }
}
