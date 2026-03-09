import SwiftUI

enum AvatarSize {
    case small
    case medium
    case large

    var dimension: CGFloat {
        switch self {
        case .small: return 28
        case .medium: return 40
        case .large: return 72
        }
    }

    var fontSize: CGFloat {
        switch self {
        case .small: return 11
        case .medium: return 16
        case .large: return 28
        }
    }
}

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
            return String(parts[0].prefix(1) + parts[1].prefix(1)).uppercased()
        }
        return String(name.prefix(2)).uppercased()
    }

    var body: some View {
        Circle()
            .fill(OffriiTheme.primary.opacity(0.15))
            .frame(width: size.dimension, height: size.dimension)
            .overlay(
                Text(initials)
                    .font(.system(size: size.fontSize, weight: .semibold))
                    .foregroundColor(OffriiTheme.primary)
            )
    }
}
