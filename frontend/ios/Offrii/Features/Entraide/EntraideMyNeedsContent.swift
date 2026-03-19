import SwiftUI

// MARK: - My Needs Content

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
                    myNeedCard(wish)
                }
            }
            .padding(.horizontal, OffriiTheme.spacingBase)
            .padding(.vertical, OffriiTheme.spacingSM)
        }
    }

    // MARK: - Card (same visual style as EntraideWishCard)

    private func myNeedCard(_ wish: MyWish) -> some View {
        Button {
            OffriiHaptics.tap()
            selectedWishId = wish.id
        } label: {
            HStack(alignment: .top, spacing: OffriiTheme.spacingMD) {
                // Category icon
                Image(systemName: categoryIcon(wish.category))
                    .font(.system(size: 18))
                    .foregroundColor(categoryColor(wish.category))
                    .frame(width: 36, height: 36)
                    .background(categoryColor(wish.category).opacity(0.12))
                    .clipShape(RoundedRectangle(cornerRadius: 8))

                // Text
                VStack(alignment: .leading, spacing: 3) {
                    Text(wish.title)
                        .font(.system(size: 15, weight: .semibold))
                        .foregroundColor(OffriiTheme.text)
                        .lineLimit(2)
                        .multilineTextAlignment(.leading)

                    if let desc = wish.description, !desc.isEmpty {
                        Text(desc)
                            .font(.system(size: 13))
                            .foregroundColor(OffriiTheme.textSecondary)
                            .lineLimit(1)
                    }

                    // Status + context
                    HStack(spacing: 4) {
                        statusBadge(wish.status)

                        if let donor = wish.matchedWithDisplayName {
                            Text("·")
                                .foregroundColor(OffriiTheme.textMuted)
                            Text(donor)
                                .foregroundColor(OffriiTheme.warning)
                        }
                    }
                    .font(.system(size: 12))

                    // Moderation note
                    if wish.status == .review || wish.status == .flagged,
                       let note = wish.moderationNote {
                        Label(note, systemImage: "exclamationmark.triangle.fill")
                            .font(.system(size: 11))
                            .foregroundColor(OffriiTheme.danger)
                    }
                }

                Spacer(minLength: 0)
            }
            .padding(OffriiTheme.spacingBase)
            .background(OffriiTheme.card)
            .cornerRadius(OffriiTheme.cornerRadiusLG)
            .shadow(color: OffriiTheme.cardShadowColor, radius: 4, x: 0, y: 2)
        }
        .buttonStyle(.plain)
        .contextMenu {
            contextMenuActions(wish)
        }
    }

    // MARK: - Context Menu Actions

    @ViewBuilder
    // swiftlint:disable:next function_body_length
    private func contextMenuActions(_ wish: MyWish) -> some View {
        switch wish.status {
        case .matched:
            Button {
                Task { await viewModel.confirmWish(id: wish.id) }
            } label: {
                Label(
                    NSLocalizedString("entraide.action.confirm", comment: ""),
                    systemImage: "checkmark.circle"
                )
            }
            Button {
                selectedWishId = wish.id
            } label: {
                Label(
                    NSLocalizedString("entraide.action.messages", comment: ""),
                    systemImage: "bubble.left"
                )
            }
            Button(role: .destructive) {
                Task { await viewModel.closeWish(id: wish.id) }
            } label: {
                Label(
                    NSLocalizedString("entraide.action.close", comment: ""),
                    systemImage: "xmark.circle"
                )
            }
        case .open:
            Button(role: .destructive) {
                Task { await viewModel.closeWish(id: wish.id) }
            } label: {
                Label(
                    NSLocalizedString("entraide.action.close", comment: ""),
                    systemImage: "xmark.circle"
                )
            }
        case .review:
            Button {
                Task { await viewModel.reopenWish(id: wish.id) }
            } label: {
                Label(
                    NSLocalizedString("entraide.action.reopen", comment: ""),
                    systemImage: "arrow.counterclockwise"
                )
            }
            Button(role: .destructive) {
                Task { await viewModel.closeWish(id: wish.id) }
            } label: {
                Label(
                    NSLocalizedString("entraide.action.close", comment: ""),
                    systemImage: "xmark.circle"
                )
            }
        default:
            EmptyView()
        }

        // Delete available except for matched and fulfilled
        if wish.status != .matched && wish.status != .fulfilled {
            Divider()
            Button(role: .destructive) {
                Task { await viewModel.deleteWish(id: wish.id) }
            } label: {
                Label(
                    NSLocalizedString("entraide.action.delete", comment: ""),
                    systemImage: "trash"
                )
            }
        }
    }

    // MARK: - Status Badge

    private func statusBadge(_ status: WishStatus) -> some View {
        let (color, label) = statusInfo(status)
        return HStack(spacing: 4) {
            Circle().fill(color).frame(width: 6, height: 6)
            Text(label)
                .font(.system(size: 12, weight: .medium))
                .foregroundColor(color)
        }
    }

    // MARK: - Helpers

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

    private func categoryColor(_ cat: WishCategory) -> Color {
        switch cat {
        case .education: return Color(red: 0.2, green: 0.4, blue: 0.85)
        case .clothing:  return Color(red: 0.7, green: 0.3, blue: 0.6)
        case .health:    return Color(red: 0.85, green: 0.3, blue: 0.35)
        case .religion:  return Color(red: 0.55, green: 0.4, blue: 0.75)
        case .home:      return Color(red: 0.9, green: 0.5, blue: 0.2)
        case .children:  return Color(red: 0.3, green: 0.7, blue: 0.6)
        case .other:     return Color(red: 0.5, green: 0.5, blue: 0.6)
        }
    }

    private func categoryIcon(_ cat: WishCategory) -> String {
        switch cat {
        case .education: return "book.fill"
        case .clothing:  return "tshirt.fill"
        case .health:    return "heart.fill"
        case .religion:  return "hands.sparkles.fill"
        case .home:      return "house.fill"
        case .children:  return "figure.and.child.holdinghands"
        case .other:     return "tag.fill"
        }
    }
}
