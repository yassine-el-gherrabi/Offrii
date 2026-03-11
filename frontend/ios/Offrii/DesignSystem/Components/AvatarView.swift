import SwiftUI

// MARK: - Avatar Size

enum AvatarSize {
    case xs
    case small
    case medium
    case large
    case xl

    var dimension: CGFloat {
        switch self {
        case .xs:     return 24
        case .small:  return 32
        case .medium: return 44
        case .large:  return 72
        case .xl:     return 96
        }
    }

    var fontSize: CGFloat {
        switch self {
        case .xs:     return 9
        case .small:  return 12
        case .medium: return 16
        case .large:  return 28
        case .xl:     return 36
        }
    }
}

// MARK: - AvatarView

struct AvatarView: View {
    let name: String?
    let size: AvatarSize

    init(_ name: String?, size: AvatarSize = .medium) {
        self.name = name
        self.size = size
    }

    private var initials: String {
        guard let name = name, !name.isEmpty else { return "?" }
        let parts = name.split(separator: " ")
        if parts.count >= 2 {
            return "\(parts[0].prefix(1))\(parts[1].prefix(1))".uppercased()
        }
        return String(name.prefix(2)).uppercased()
    }

    var body: some View {
        Circle()
            .fill(
                LinearGradient(
                    colors: [OffriiTheme.primary.opacity(0.2), OffriiTheme.accent.opacity(0.15)],
                    startPoint: .topLeading,
                    endPoint: .bottomTrailing
                )
            )
            .frame(width: size.dimension, height: size.dimension)
            .overlay(
                Text(initials)
                    .font(.system(size: size.fontSize, weight: .semibold))
                    .foregroundColor(OffriiTheme.primary)
            )
    }
}
