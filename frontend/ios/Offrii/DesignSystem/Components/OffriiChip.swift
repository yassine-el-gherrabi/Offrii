import SwiftUI

// MARK: - OffriiChip

struct OffriiChip: View {
    let title: String
    let isSelected: Bool
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            Text(title)
                .font(OffriiTypography.footnote)
                .fontWeight(isSelected ? .semibold : .regular)
                .foregroundColor(isSelected ? .white : OffriiTheme.textSecondary)
                .padding(.horizontal, OffriiTheme.spacingMD)
                .padding(.vertical, OffriiTheme.spacingSM)
                .background(isSelected ? OffriiTheme.primary : OffriiTheme.cardSurface)
                .cornerRadius(OffriiTheme.cornerRadiusXL)
        }
        .buttonStyle(.plain)
        .animation(OffriiTheme.defaultAnimation, value: isSelected)
    }
}

// MARK: - Chip Group (convenience)

struct OffriiChipGroup: View {
    let items: [String]
    @Binding var selectedItem: String?

    var body: some View {
        ScrollView(.horizontal, showsIndicators: false) {
            HStack(spacing: OffriiTheme.spacingSM) {
                ForEach(items, id: \.self) { item in
                    OffriiChip(
                        title: item,
                        isSelected: selectedItem == item,
                        action: {
                            if selectedItem == item {
                                selectedItem = nil
                            } else {
                                selectedItem = item
                            }
                        }
                    )
                }
            }
            .padding(.horizontal, OffriiTheme.spacingMD)
        }
    }
}

// MARK: - Multi-Select Chip Group

struct OffriiChipGroupMulti: View {
    let items: [String]
    @Binding var selectedItems: Set<String>

    var body: some View {
        ScrollView(.horizontal, showsIndicators: false) {
            HStack(spacing: OffriiTheme.spacingSM) {
                ForEach(items, id: \.self) { item in
                    OffriiChip(
                        title: item,
                        isSelected: selectedItems.contains(item),
                        action: {
                            if selectedItems.contains(item) {
                                selectedItems.remove(item)
                            } else {
                                selectedItems.insert(item)
                            }
                        }
                    )
                }
            }
            .padding(.horizontal, OffriiTheme.spacingMD)
        }
    }
}

// MARK: - Preview

#if DEBUG
struct OffriiChip_Previews: PreviewProvider {
    struct PreviewWrapper: View {
        @State private var selected: String? = "Maison"
        @State private var multiSelected: Set<String> = ["Sport"]
        private let categories = ["Tout", "Maison", "Sport", "Tech", "Mode", "Cuisine"]

        var body: some View {
            VStack(alignment: .leading, spacing: OffriiTheme.spacingLG) {
                Text("Single Select")
                    .font(OffriiTypography.headline)
                    .foregroundColor(OffriiTheme.text)
                OffriiChipGroup(items: categories, selectedItem: $selected)

                Text("Multi Select")
                    .font(OffriiTypography.headline)
                    .foregroundColor(OffriiTheme.text)
                OffriiChipGroupMulti(items: categories, selectedItems: $multiSelected)
            }
            .padding(.vertical, OffriiTheme.spacingLG)
            .background(OffriiTheme.card)
        }
    }

    static var previews: some View {
        PreviewWrapper()
            .previewLayout(.sizeThatFits)
    }
}
#endif
