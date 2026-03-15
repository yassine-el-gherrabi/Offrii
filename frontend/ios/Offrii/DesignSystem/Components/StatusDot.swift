import SwiftUI

// MARK: - StatusDot Variant

enum StatusDotVariant {
    case open
    case matched
    case fulfilled
    case closed
    case reserved
    case priority(Int)

    var color: Color {
        switch self {
        case .open:        return OffriiTheme.success
        case .matched:     return OffriiTheme.accent
        case .fulfilled:   return OffriiTheme.success
        case .closed:      return OffriiTheme.textMuted
        case .reserved:    return OffriiTheme.accent
        case .priority(let level):
            switch level {
            case 3:  return OffriiTheme.danger
            case 2:  return OffriiTheme.accent
            default: return .clear
            }
        }
    }

    var icon: String? {
        switch self {
        case .fulfilled: return "checkmark"
        case .reserved:  return "lock.fill"
        default:         return nil
        }
    }

    var label: String? {
        switch self {
        case .open:      return NSLocalizedString("entraide.status.open", comment: "")
        case .matched:   return NSLocalizedString("entraide.status.matched", comment: "")
        case .fulfilled: return NSLocalizedString("entraide.status.fulfilled", comment: "")
        case .closed:    return NSLocalizedString("entraide.status.closed", comment: "")
        case .reserved:  return NSLocalizedString("wishlist.reserved", comment: "")
        case .priority:  return nil
        }
    }
}

// MARK: - StatusDot

struct StatusDot: View {
    let variant: StatusDotVariant
    var showLabel: Bool = false
    var size: CGFloat = 8

    var body: some View {
        if case .priority(let level) = variant, level <= 1 {
            EmptyView()
        } else {
            HStack(spacing: 4) {
                if let icon = variant.icon {
                    Image(systemName: icon)
                        .font(.system(size: size * 0.8, weight: .bold))
                        .foregroundColor(variant.color)
                } else {
                    Circle()
                        .fill(variant.color)
                        .frame(width: size, height: size)
                }

                if showLabel, let label = variant.label {
                    Text(label)
                        .font(OffriiTypography.caption)
                        .foregroundColor(variant.color)
                }
            }
        }
    }
}
