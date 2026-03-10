import SwiftUI

// MARK: - WishStatusBadge

struct WishStatusBadge: View {
    let status: WishStatus

    var body: some View {
        HStack(spacing: OffriiTheme.spacingXS) {
            Circle()
                .fill(dotColor)
                .frame(width: 6, height: 6)

            Text(statusLabel)
                .font(OffriiTypography.caption2)
                .fontWeight(.semibold)
                .foregroundColor(labelColor)
        }
        .padding(.horizontal, OffriiTheme.spacingSM)
        .padding(.vertical, 3)
        .background(badgeBackground)
        .cornerRadius(OffriiTheme.cornerRadiusSM)
    }

    // MARK: - Computed Properties

    private var statusLabel: String {
        switch status {
        case .open:      return NSLocalizedString("entraide.status.open", comment: "")
        case .matched:   return NSLocalizedString("entraide.status.matched", comment: "")
        case .fulfilled: return NSLocalizedString("entraide.status.fulfilled", comment: "")
        case .closed:    return NSLocalizedString("entraide.status.closed", comment: "")
        case .pending:   return NSLocalizedString("entraide.status.pending", comment: "")
        case .review:    return NSLocalizedString("entraide.status.review", comment: "")
        case .flagged:   return NSLocalizedString("entraide.status.flagged", comment: "")
        case .rejected:  return NSLocalizedString("entraide.status.rejected", comment: "")
        }
    }

    private var dotColor: Color {
        switch status {
        case .open:      return OffriiTheme.success
        case .matched:   return OffriiTheme.accent
        case .fulfilled: return OffriiTheme.primary
        case .closed:    return OffriiTheme.textMuted
        case .pending, .review: return OffriiTheme.accent
        case .flagged, .rejected: return OffriiTheme.danger
        }
    }

    private var labelColor: Color {
        switch status {
        case .open:      return OffriiTheme.success
        case .matched:   return OffriiTheme.accent
        case .fulfilled: return OffriiTheme.primary
        case .closed:    return OffriiTheme.textMuted
        case .pending, .review: return OffriiTheme.accent
        case .flagged, .rejected: return OffriiTheme.danger
        }
    }

    private var badgeBackground: Color {
        dotColor.opacity(0.12)
    }
}
