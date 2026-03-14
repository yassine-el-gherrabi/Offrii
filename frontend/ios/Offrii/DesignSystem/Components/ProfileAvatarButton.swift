import SwiftUI

/// Small avatar button for navigating to ProfileView from any screen header.
struct ProfileAvatarButton: View {
    let initials: String
    var showBadge: Bool = false

    var body: some View {
        ZStack(alignment: .topTrailing) {
            // Avatar circle with initials
            Text(initials)
                .font(.system(size: 13, weight: .semibold))
                .foregroundColor(.white)
                .frame(width: 32, height: 32)
                .background(Color.white.opacity(0.25))
                .clipShape(Circle())
                .overlay(
                    Circle()
                        .strokeBorder(Color.white.opacity(0.4), lineWidth: 1)
                )

            // Badge dot
            if showBadge {
                Circle()
                    .fill(OffriiTheme.accent)
                    .frame(width: 10, height: 10)
                    .overlay(
                        Circle()
                            .strokeBorder(Color.white, lineWidth: 1.5)
                    )
                    .offset(x: 2, y: -2)
            }
        }
    }

    /// Extract initials from a display name (e.g. "Yassine" → "Y", "Marie Dupont" → "MD")
    static func initials(from name: String?) -> String {
        guard let name = name, !name.isEmpty else { return "?" }
        let parts = name.split(separator: " ")
        if parts.count >= 2 {
            return String(parts[0].prefix(1) + parts[1].prefix(1)).uppercased()
        }
        return String(name.prefix(1)).uppercased()
    }
}
