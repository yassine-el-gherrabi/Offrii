import SwiftUI

// MARK: - My Needs Content

struct EntraideMyNeedsContent: View {
    var viewModel: EntraideMyNeedsViewModel
    @Binding var selectedWishId: UUID?
    @Binding var showCreateSheet: Bool
    @State private var wishToClose: UUID?
    @State private var wishToDelete: UUID?
    @State private var wishToEdit: MyWish?

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
            .alert(
                NSLocalizedString("entraide.close.confirmTitle", comment: ""),
                isPresented: Binding(
                    get: { wishToClose != nil },
                    set: { if !$0 { wishToClose = nil } }
                )
            ) {
                Button(NSLocalizedString("common.cancel", comment: ""), role: .cancel) {
                    wishToClose = nil
                }
                Button(NSLocalizedString("entraide.action.close", comment: ""), role: .destructive) {
                    if let id = wishToClose {
                        Task { await viewModel.closeWish(id: id) }
                    }
                    wishToClose = nil
                }
            } message: {
                Text(NSLocalizedString("entraide.close.confirmMessage", comment: ""))
            }
            .alert(
                NSLocalizedString("entraide.delete.confirmTitle", comment: ""),
                isPresented: Binding(
                    get: { wishToDelete != nil },
                    set: { if !$0 { wishToDelete = nil } }
                )
            ) {
                Button(NSLocalizedString("common.cancel", comment: ""), role: .cancel) {
                    wishToDelete = nil
                }
                Button(NSLocalizedString("entraide.action.delete", comment: ""), role: .destructive) {
                    if let id = wishToDelete {
                        Task { await viewModel.deleteWish(id: id) }
                    }
                    wishToDelete = nil
                }
            } message: {
                Text(NSLocalizedString("entraide.delete.confirmMessage", comment: ""))
            }
            .sheet(item: $wishToEdit, onDismiss: {
                Task { await viewModel.loadMyWishes() }
            }) { wish in
                CreateWishSheet(
                    editingWishId: wish.id,
                    editingTitle: wish.title,
                    editingDescription: wish.description,
                    editingCategory: wish.category,
                    editingImageUrl: wish.imageUrl,
                    editingLinks: wish.links
                )
                .presentationDetents([.large])
            }
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
                Image(systemName: wish.category.icon)
                    .font(.system(size: 18))
                    .foregroundColor(wish.category.color)
                    .frame(width: 36, height: 36)
                    .background(wish.category.color.opacity(0.12))
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

                        if wish.status == .matched, let matchedAt = wish.matchedAt {
                            Text("·")
                                .foregroundColor(OffriiTheme.textMuted)
                            Text(matchedAt, style: .relative)
                                .foregroundColor(OffriiTheme.textMuted)
                        }

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
                        Label(
                            humanReadableModerationNote(note),
                            systemImage: "exclamationmark.triangle.fill"
                        )
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
        // Edit for open/review
        if wish.status == .open || wish.status == .review {
            Button {
                wishToEdit = wish
            } label: {
                Label(
                    NSLocalizedString("entraide.action.edit", comment: ""),
                    systemImage: "pencil"
                )
            }
        }

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
                wishToClose = wish.id
            } label: {
                Label(
                    NSLocalizedString("entraide.action.close", comment: ""),
                    systemImage: "xmark.circle"
                )
            }
        case .open:
            Button(role: .destructive) {
                wishToClose = wish.id
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
                wishToClose = wish.id
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
                wishToDelete = wish.id
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

    private func humanReadableModerationNote(_ note: String) -> String {
        // Map OpenAI category names to localized labels
        let categoryMap: [String: String] = [
            "harassment": NSLocalizedString("moderation.harassment", comment: ""),
            "harassment/threatening": NSLocalizedString("moderation.harassment_threatening", comment: ""),
            "hate": NSLocalizedString("moderation.hate", comment: ""),
            "hate/threatening": NSLocalizedString("moderation.hate_threatening", comment: ""),
            "sexual": NSLocalizedString("moderation.sexual", comment: ""),
            "sexual/minors": NSLocalizedString("moderation.sexual_minors", comment: ""),
            "violence": NSLocalizedString("moderation.violence", comment: ""),
            "violence/graphic": NSLocalizedString("moderation.violence_graphic", comment: ""),
            "self-harm": NSLocalizedString("moderation.self_harm", comment: ""),
            "self-harm/intent": NSLocalizedString("moderation.self_harm_intent", comment: ""),
            "self-harm/instructions": NSLocalizedString("moderation.self_harm_instructions", comment: ""),
            "illicit": NSLocalizedString("moderation.illicit", comment: ""),
            "illicit/violent": NSLocalizedString("moderation.illicit_violent", comment: ""),
        ]

        // "flagged categories: harassment, violence" → extract categories
        if note.hasPrefix("flagged categories: ") {
            let raw = note.replacingOccurrences(of: "flagged categories: ", with: "")
            let categories = raw.split(separator: ",").map { $0.trimmingCharacters(in: .whitespaces) }
            let mapped = categories.map { categoryMap[$0] ?? $0 }
            return NSLocalizedString("moderation.prefix", comment: "") + mapped.joined(separator: ", ")
        }

        // "moderation service unavailable after retries"
        if note.contains("unavailable") {
            return NSLocalizedString("moderation.unavailable", comment: "")
        }

        // "content flagged by moderation"
        if note.contains("flagged") {
            return NSLocalizedString("moderation.generic", comment: "")
        }

        return note
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
