import SwiftUI

// MARK: - My Needs Content (no ScrollView — parent handles it)

struct EntraideMyNeedsContent: View {
    var viewModel: EntraideMyNeedsViewModel
    @Binding var selectedWishId: UUID?
    @Binding var showCreateSheet: Bool

    var body: some View {
        if viewModel.isLoading && viewModel.wishes.isEmpty {
            LazyVStack(spacing: OffriiTheme.spacingSM) {
                ForEach(0..<4, id: \.self) { _ in
                    SkeletonRow()
                }
            }
            .padding(.horizontal, OffriiTheme.spacingBase)
            .padding(.top, OffriiTheme.spacingBase)
        } else if viewModel.wishes.isEmpty {
            VStack(spacing: OffriiTheme.spacingBase) {
                Spacer().frame(height: 40)
                OffriiEmptyState(
                    icon: "tray",
                    title: NSLocalizedString("entraide.myWishes.empty", comment: ""),
                    subtitle: NSLocalizedString("entraide.myWishes.emptySubtitle", comment: ""),
                    ctaTitle: NSLocalizedString("entraide.fab.publish", comment: ""),
                    ctaAction: { showCreateSheet = true }
                )
                Spacer()
            }
        } else {
            LazyVStack(spacing: OffriiTheme.spacingSM) {
                ForEach(viewModel.wishes) { wish in
                    myWishRow(wish)
                }
            }
            .padding(.horizontal, OffriiTheme.spacingBase)
            .padding(.vertical, OffriiTheme.spacingSM)
        }
    }

    // MARK: - Wish Row

    private func myWishRow(_ wish: MyWish) -> some View {
        Button {
            OffriiHaptics.tap()
            selectedWishId = wish.id
        } label: {
            VStack(alignment: .leading, spacing: OffriiTheme.spacingSM) {
                HStack {
                    VStack(alignment: .leading, spacing: 2) {
                        Text(wish.title)
                            .font(OffriiTypography.body)
                            .fontWeight(.semibold)
                            .foregroundColor(OffriiTheme.text)
                            .lineLimit(2)

                        Text(wish.category.label)
                            .font(OffriiTypography.caption)
                            .foregroundColor(OffriiTheme.textMuted)
                    }

                    Spacer()

                    statusBadge(wish.status)
                }

                contextInfo(wish)
                actionButtons(wish)
            }
            .padding(OffriiTheme.spacingBase)
            .background(OffriiTheme.card)
            .cornerRadius(OffriiTheme.cornerRadiusLG)
            .shadow(color: OffriiTheme.cardShadowColor, radius: 4, x: 0, y: 2)
        }
        .buttonStyle(.plain)
    }

    @ViewBuilder
    private func statusBadge(_ status: WishStatus) -> some View {
        let (color, label) = statusInfo(status)
        HStack(spacing: 4) {
            Circle().fill(color).frame(width: 6, height: 6)
            Text(label)
                .font(.system(size: 11, weight: .medium))
                .foregroundColor(color)
        }
        .padding(.horizontal, 8)
        .padding(.vertical, 4)
        .background(color.opacity(0.1))
        .cornerRadius(OffriiTheme.cornerRadiusSM)
    }

    @ViewBuilder
    private func contextInfo(_ wish: MyWish) -> some View {
        if let donor = wish.matchedWithDisplayName {
            Label(
                String(format: NSLocalizedString("entraide.detail.matchedBy", comment: ""), donor),
                systemImage: "person.fill.checkmark"
            )
            .font(OffriiTypography.caption)
            .foregroundColor(OffriiTheme.warning)
        }

        if wish.status == .review || wish.status == .flagged,
           let note = wish.moderationNote {
            Label(note, systemImage: "exclamationmark.triangle.fill")
                .font(OffriiTypography.caption)
                .foregroundColor(OffriiTheme.danger)
        }

        if wish.reportCount > 0 && (wish.status == .review || wish.status == .open) {
            Label(
                String(format: NSLocalizedString("entraide.myWishes.reports", comment: ""), wish.reportCount),
                systemImage: "flag.fill"
            )
            .font(OffriiTypography.caption)
            .foregroundColor(OffriiTheme.textMuted)
        }
    }

    @ViewBuilder
    private func actionButtons(_ wish: MyWish) -> some View {
        HStack(spacing: OffriiTheme.spacingSM) {
            switch wish.status {
            case .open:
                actionChip(
                    NSLocalizedString("entraide.action.close", comment: ""),
                    icon: "xmark.circle", color: OffriiTheme.danger
                ) {
                    Task { await viewModel.closeWish(id: wish.id) }
                }
            case .matched:
                actionChip(
                    NSLocalizedString("entraide.action.confirm", comment: ""),
                    icon: "checkmark.circle.fill", color: OffriiTheme.success
                ) {
                    Task { await viewModel.confirmWish(id: wish.id) }
                }
                actionChip(
                    NSLocalizedString("entraide.action.messages", comment: ""),
                    icon: "bubble.left.fill", color: OffriiTheme.primary
                ) {
                    selectedWishId = wish.id
                }
                actionChip(
                    NSLocalizedString("entraide.action.close", comment: ""),
                    icon: "xmark.circle", color: OffriiTheme.danger
                ) {
                    Task { await viewModel.closeWish(id: wish.id) }
                }
            case .review:
                actionChip(
                    NSLocalizedString("entraide.action.close", comment: ""),
                    icon: "xmark.circle", color: OffriiTheme.danger
                ) {
                    Task { await viewModel.closeWish(id: wish.id) }
                }
            default:
                EmptyView()
            }
        }
    }

    private func actionChip(
        _ label: String, icon: String, color: Color, action: @escaping () -> Void
    ) -> some View {
        Button(action: action) {
            Label(label, systemImage: icon)
                .font(.system(size: 12, weight: .medium))
                .foregroundColor(color)
                .padding(.horizontal, OffriiTheme.spacingSM)
                .padding(.vertical, OffriiTheme.spacingXS)
                .background(color.opacity(0.1))
                .cornerRadius(OffriiTheme.cornerRadiusXL)
        }
        .buttonStyle(.plain)
    }

    private func statusInfo(_ status: WishStatus) -> (Color, String) {
        switch status {
        case .open:      return (OffriiTheme.success, NSLocalizedString("entraide.status.open", comment: ""))
        case .matched:   return (OffriiTheme.warning, NSLocalizedString("entraide.status.matched", comment: ""))
        case .fulfilled: return (OffriiTheme.primary, NSLocalizedString("entraide.status.fulfilled", comment: ""))
        case .closed:    return (OffriiTheme.textMuted, NSLocalizedString("entraide.status.closed", comment: ""))
        case .pending:   return (OffriiTheme.textMuted, NSLocalizedString("entraide.status.pending", comment: ""))
        case .review:    return (OffriiTheme.warning, NSLocalizedString("entraide.status.review", comment: ""))
        case .flagged:   return (OffriiTheme.danger, NSLocalizedString("entraide.status.flagged", comment: ""))
        case .rejected:  return (OffriiTheme.danger, NSLocalizedString("entraide.status.rejected", comment: ""))
        }
    }
}
