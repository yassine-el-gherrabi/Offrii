import SwiftUI

// MARK: - OffriiChip

struct OffriiChip: View {
    let title: String
    let isSelected: Bool
    var backgroundColor: Color?
    var textColor: Color?
    var badgeCount: Int = 0
    let action: () -> Void

    @State private var isPressed = false

    var body: some View {
        Button(action: {
            OffriiHaptics.selection()
            action()
        }) {
            HStack(spacing: 4) {
                Text(title)
                    .font(OffriiTypography.footnote)
                    .fontWeight(isSelected ? .semibold : .regular)
                    .foregroundColor(chipTextColor)

                if badgeCount > 0 {
                    Text("\(badgeCount)")
                        .font(.system(size: 10, weight: .bold))
                        .foregroundColor(.white)
                        .padding(.horizontal, 5)
                        .padding(.vertical, 1)
                        .background(OffriiTheme.primary)
                        .clipShape(Capsule())
                }
            }
            .padding(.horizontal, OffriiTheme.spacingMD)
            .padding(.vertical, OffriiTheme.spacingSM)
            .background(chipBackground)
            .cornerRadius(OffriiTheme.cornerRadiusSM)
        }
        .buttonStyle(.plain)
        .scaleEffect(isPressed ? 1.05 : 1.0)
        .animation(OffriiAnimation.snappy, value: isSelected)
        .animation(OffriiAnimation.micro, value: isPressed)
        .pressEvents {
            isPressed = true
        } onRelease: {
            isPressed = false
        }
    }

    private var chipBackground: Color {
        if let backgroundColor, isSelected {
            return backgroundColor
        }
        return isSelected ? OffriiTheme.primary : OffriiTheme.surface
    }

    private var chipTextColor: Color {
        if let textColor, isSelected {
            return textColor
        }
        return isSelected ? .white : OffriiTheme.textSecondary
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
            .padding(.horizontal, OffriiTheme.spacingBase)
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
            .padding(.horizontal, OffriiTheme.spacingBase)
        }
    }
}
