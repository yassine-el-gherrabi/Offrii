import SwiftUI

// MARK: - Category Visual Style

/// Maps backend category icon names to SF Symbols and gradient colors.
/// Used by both Envies (items with UUID categories) and Entraide (WishCategory enum).
enum CategoryStyle {
    case tech
    case mode
    case maison
    case loisirs
    case sante
    case autre
    case unknown

    /// Initialize from the backend `icon` field of a CategoryResponse.
    init(icon: String?) {
        switch icon {
        case "laptop":  self = .tech
        case "tshirt":  self = .mode
        case "home":    self = .maison
        case "gamepad": self = .loisirs
        case "heart":   self = .sante
        case "tag":     self = .autre
        default:        self = .unknown
        }
    }

    var sfSymbol: String {
        switch self {
        case .tech:    return "laptopcomputer"
        case .mode:    return "tshirt.fill"
        case .maison:  return "house.fill"
        case .loisirs: return "gamecontroller.fill"
        case .sante:   return "heart.fill"
        case .autre:   return "tag.fill"
        case .unknown: return "gift.fill"
        }
    }

    var gradient: [Color] {
        switch self {
        case .tech:    return [Color(red: 0.2, green: 0.5, blue: 0.9), Color(red: 0.4, green: 0.7, blue: 1.0)]
        case .mode:    return [Color(red: 0.6, green: 0.3, blue: 0.8), Color(red: 0.8, green: 0.5, blue: 1.0)]
        case .maison:  return [Color(red: 0.9, green: 0.5, blue: 0.2), Color(red: 1.0, green: 0.7, blue: 0.4)]
        case .loisirs: return [Color(red: 0.3, green: 0.7, blue: 0.6), Color(red: 0.5, green: 0.9, blue: 0.8)]
        case .sante:   return [Color(red: 0.3, green: 0.7, blue: 0.4), Color(red: 0.5, green: 0.9, blue: 0.6)]
        case .autre:   return [Color(red: 0.5, green: 0.5, blue: 0.6), Color(red: 0.7, green: 0.7, blue: 0.8)]
        case .unknown: return [OffriiTheme.primary.opacity(0.7), OffriiTheme.accent.opacity(0.5)]
        }
    }

    var chipColor: Color {
        switch self {
        case .tech:    return Color(red: 0.2, green: 0.5, blue: 0.9)
        case .mode:    return Color(red: 0.6, green: 0.3, blue: 0.8)
        case .maison:  return Color(red: 0.9, green: 0.5, blue: 0.2)
        case .loisirs: return Color(red: 0.3, green: 0.7, blue: 0.6)
        case .sante:   return Color(red: 0.3, green: 0.7, blue: 0.4)
        case .autre:   return Color(red: 0.5, green: 0.5, blue: 0.6)
        case .unknown: return OffriiTheme.primary
        }
    }
}
