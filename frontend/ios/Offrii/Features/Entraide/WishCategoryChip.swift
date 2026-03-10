import SwiftUI

// MARK: - Category Color Helper

extension WishCategory {
    var backgroundColor: Color {
        switch self {
        case .education: return OffriiTheme.categoryEducationBg
        case .clothing:  return OffriiTheme.categoryClothingBg
        case .health:    return OffriiTheme.categoryHealthBg
        case .religion:  return OffriiTheme.categoryReligionBg
        case .home:      return OffriiTheme.categoryHomeBg
        case .children:  return OffriiTheme.categoryChildrenBg
        case .other:     return OffriiTheme.categoryOtherBg
        }
    }

    var textColor: Color {
        switch self {
        case .education: return OffriiTheme.categoryEducationText
        case .clothing:  return OffriiTheme.categoryClothingText
        case .health:    return OffriiTheme.categoryHealthText
        case .religion:  return OffriiTheme.categoryReligionText
        case .home:      return OffriiTheme.categoryHomeText
        case .children:  return OffriiTheme.categoryChildrenText
        case .other:     return OffriiTheme.categoryOtherText
        }
    }

    var chipLabel: String {
        "\(emoji) \(label)"
    }
}

// MARK: - WishCategoryChip

struct WishCategoryChip: View {
    let category: WishCategory

    var body: some View {
        Text(category.chipLabel)
            .font(OffriiTypography.caption)
            .fontWeight(.medium)
            .foregroundColor(category.textColor)
            .padding(.horizontal, OffriiTheme.spacingSM)
            .padding(.vertical, OffriiTheme.spacingXS)
            .background(category.backgroundColor)
            .cornerRadius(OffriiTheme.cornerRadiusSM)
    }
}
