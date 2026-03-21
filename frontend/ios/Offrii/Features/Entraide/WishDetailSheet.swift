// swiftlint:disable file_length
import NukeUI
import SwiftUI

// MARK: - Wish Detail Sheet

// swiftlint:disable:next type_body_length
struct WishDetailSheet: View {
    let wishId: UUID
    var onOpenMessages: (() -> Void)?
    var onReport: (() -> Void)?
    var onAction: (() -> Void)?

    @Environment(\.dismiss) private var dismiss
    @Environment(AuthManager.self) private var authManager
    @State private var viewModel = WishDetailViewModel()
    @State private var showOfferSheet = false
    @State private var offerMessage = ""
    @State private var showCloseConfirm = false
    @State private var showDeleteConfirm = false
    @State private var showEditSheet = false
    @State private var showConfirmFulfillment = false
    @State private var showWithdrawConfirm = false
    @State private var showCelebration = false
    @State private var showMessagesSheet = false
    @State private var showReportSheet = false

    private var wish: WishDetail? { viewModel.wish }
    private var isMine: Bool { wish?.isMine ?? false }
    private var isMatchedByMe: Bool { wish?.isMatchedByMe ?? false }

    var body: some View {
        NavigationStack {
            ZStack {
                OffriiTheme.background.ignoresSafeArea()

                if showCelebration {
                    VStack(spacing: OffriiTheme.spacingLG) {
                        Image(systemName: "heart.circle.fill")
                            .font(.system(size: 64))
                            .foregroundColor(OffriiTheme.warning)
                        Text(NSLocalizedString("entraide.fulfill.celebration", comment: ""))
                            .font(OffriiTypography.headline)
                            .foregroundColor(OffriiTheme.text)
                            .multilineTextAlignment(.center)
                    }
                    .transition(.opacity)
                } else if viewModel.isLoading && wish == nil {
                    VStack(spacing: OffriiTheme.spacingBase) {
                        SkeletonRow(height: 120)
                        SkeletonRow()
                        SkeletonRow()
                    }
                    .padding(.top, OffriiTheme.spacingLG)
                } else if let wish {
                    wishContent(wish)
                }
            }
            .animation(OffriiAnimation.gentle, value: showCelebration)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button { dismiss() } label: {
                        Image(systemName: "xmark")
                            .font(.system(size: 16, weight: .semibold))
                            .foregroundColor(OffriiTheme.textMuted)
                    }
                }
            }
        }
        .sheet(isPresented: $showMessagesSheet) {
            WishMessagesSheet(wishId: wishId)
                .presentationDetents([.large])
        }
        .sheet(isPresented: $showReportSheet, onDismiss: {
            Task { await viewModel.loadWish(id: wishId) }
            onAction?()
        }) {
            ReportWishSheet(wishId: wishId)
                .presentationDetents([.medium])
        }
        .sheet(isPresented: $showEditSheet, onDismiss: {
            Task { await viewModel.loadWish(id: wishId) }
            onAction?()
        }) {
            if let wish = viewModel.wish {
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
        .task {
            await viewModel.loadWish(id: wishId)
        }
        .onChange(of: viewModel.actionSuccess) { _, newValue in
            if newValue != nil {
                onAction?()
                // Don't dismiss during fulfillment celebration — it has its own dismiss
                if !showCelebration {
                    dismiss()
                }
            }
        }
    }

    // MARK: - Content

    // swiftlint:disable:next function_body_length
    private func wishContent(_ wish: WishDetail) -> some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 0) {
                // Hero image
                heroImage(wish)

                VStack(alignment: .leading, spacing: OffriiTheme.spacingMD) {
                    // Title
                    Text(wish.title)
                        .font(OffriiTypography.headline)
                        .foregroundColor(OffriiTheme.text)

                    // Category + Status
                    HStack(spacing: OffriiTheme.spacingSM) {
                        categoryChip(wish.category)
                        statusBadge(wish.status)
                        Spacer()
                    }

                    // Author + date
                    HStack(spacing: OffriiTheme.spacingXS) {
                        Image(systemName: "person.fill")
                            .font(.system(size: 12))
                            .foregroundColor(OffriiTheme.textMuted)
                        Text(wish.displayName ?? NSLocalizedString("entraide.anonymous", comment: ""))
                            .font(OffriiTypography.caption)
                            .foregroundColor(OffriiTheme.textSecondary)
                        Text("·")
                            .foregroundColor(OffriiTheme.textMuted)
                        Text(wish.createdAt, style: .relative)
                            .font(OffriiTypography.caption)
                            .foregroundColor(OffriiTheme.textMuted)
                    }

                    // Description
                    if let description = wish.description, !description.isEmpty {
                        Text(description)
                            .font(OffriiTypography.body)
                            .foregroundColor(OffriiTheme.textSecondary)
                            .fixedSize(horizontal: false, vertical: true)
                    }

                    // Links
                    if let links = wish.links, !links.isEmpty {
                        VStack(alignment: .leading, spacing: OffriiTheme.spacingXS) {
                            ForEach(links, id: \.self) { link in
                                if let url = URL(string: link) {
                                    Link(destination: url) {
                                        HStack(spacing: 4) {
                                            Image(systemName: "link")
                                                .font(.system(size: 12))
                                            Text(link)
                                                .font(OffriiTypography.caption)
                                                .lineLimit(1)
                                        }
                                        .foregroundColor(OffriiTheme.primary)
                                    }
                                }
                            }
                        }
                    }

                    // Matched info
                    if wish.status == .matched, let donor = wish.matchedWithDisplayName {
                        Label(
                            String(format: NSLocalizedString("entraide.detail.matchedBy", comment: ""), donor),
                            systemImage: "person.fill.checkmark"
                        )
                        .font(OffriiTypography.subheadline)
                        .foregroundColor(OffriiTheme.warning)
                        .padding(OffriiTheme.spacingSM)
                        .background(OffriiTheme.warning.opacity(0.1))
                        .cornerRadius(OffriiTheme.cornerRadiusMD)
                    }

                    Divider()

                    // Actions
                    actionButtons(wish)

                    // Error display

                }
                .padding(OffriiTheme.spacingLG)
            }
        }
    }

    // MARK: - Hero Image

    @ViewBuilder
    private func heroImage(_ wish: WishDetail) -> some View {
        let imgUrl = wish.imageUrl.flatMap { URL(string: $0) }
            ?? wish.ogImageUrl.flatMap { URL(string: $0) }
        if let url = imgUrl {
            LazyImage(url: url) { state in
                if let image = state.image {
                    image
                        .resizable()
                        .aspectRatio(contentMode: .fill)
                        .frame(maxWidth: .infinity)
                        .frame(height: 200)
                        .clipped()
                } else {
                    gradientPlaceholder(wish.category)
                }
            }
            .frame(height: 200)
        } else {
            gradientPlaceholder(wish.category)
        }
    }

    private func gradientPlaceholder(_ category: WishCategory) -> some View {
        return LinearGradient(
            colors: category.gradient,
            startPoint: .topLeading,
            endPoint: .bottomTrailing
        )
        .frame(height: 200)
        .overlay(
            Image(systemName: category.icon)
                .font(.system(size: 48, weight: .light))
                .foregroundColor(.white.opacity(0.5))
        )
    }

    // MARK: - Category Chip

    private func categoryChip(_ category: WishCategory) -> some View {
        let color = category.color
        return HStack(spacing: 4) {
            Image(systemName: category.icon)
                .font(.system(size: 11))
            Text(category.label)
                .font(.system(size: 12, weight: .medium))
        }
        .foregroundColor(color)
        .padding(.horizontal, 8)
        .padding(.vertical, 4)
        .background(color.opacity(0.12))
        .cornerRadius(OffriiTheme.cornerRadiusSM)
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

    // MARK: - Actions

    @ViewBuilder
    private func actionButtons(_ wish: WishDetail) -> some View {
        VStack(spacing: OffriiTheme.spacingSM) {
            visitorActions(wish)
            ownerMatchedActions(wish)
            donorMatchedActions(wish)
            ownerOpenActions(wish)
            reportAction(wish)
        }
    }

    private var isUserEligible: Bool {
        EntraideEligibility(user: authManager.currentUser).isEligible
    }

    @ViewBuilder
    private func visitorActions(_ wish: WishDetail) -> some View {
        if wish.status == .open && !isMine && !(wish.hasReported ?? false) && isUserEligible {
            OffriiButton(
                NSLocalizedString("entraide.offer.cta", comment: ""),
                variant: .primary,
                isLoading: viewModel.isActioning
            ) {
                offerMessage = ""
                showOfferSheet = true
            }
            .sheet(isPresented: $showOfferSheet) {
                offerConfirmSheet(wish)
                    .presentationDetents([.height(280)])
                    .presentationDragIndicator(.visible)
            }
        }
    }

    private func offerConfirmSheet(_ wish: WishDetail) -> some View {
        VStack(alignment: .leading, spacing: OffriiTheme.spacingMD) {
            Text(NSLocalizedString("entraide.offer.confirmTitle", comment: ""))
                .font(OffriiTypography.headline)
                .foregroundColor(OffriiTheme.text)

            Text(NSLocalizedString("entraide.offer.confirmMessage", comment: ""))
                .font(OffriiTypography.caption)
                .foregroundColor(OffriiTheme.textSecondary)

            LimitedTextEditor(
                placeholder: NSLocalizedString("entraide.offer.messagePlaceholder", comment: ""),
                text: $offerMessage
            )

            HStack(spacing: OffriiTheme.spacingSM) {
                OffriiButton(NSLocalizedString("common.cancel", comment: ""), variant: .ghost) {
                    showOfferSheet = false
                }
                OffriiButton(
                    NSLocalizedString("entraide.offer.confirmAction", comment: ""),
                    variant: .primary,
                    isLoading: viewModel.isActioning
                ) {
                    Task {
                        if await viewModel.offer(id: wish.id) {
                            // Send initial message if provided
                            let msg = offerMessage.trimmingCharacters(in: .whitespacesAndNewlines)
                            if !msg.isEmpty {
                                _ = try? await WishMessageService.shared.sendMessage(
                                    wishId: wish.id, body: msg
                                )
                            }
                            showOfferSheet = false
                        }
                    }
                }
            }
        }
        .padding(OffriiTheme.spacingLG)
    }

    @ViewBuilder
    private func ownerMatchedActions(_ wish: WishDetail) -> some View {
        if wish.status == .matched && isMine {
            OffriiButton(
                NSLocalizedString("entraide.action.confirm", comment: ""),
                variant: .primary,
                isLoading: viewModel.isActioning
            ) {
                showConfirmFulfillment = true
            }
            .alert(
                NSLocalizedString("entraide.fulfill.confirmTitle", comment: ""),
                isPresented: $showConfirmFulfillment
            ) {
                Button(NSLocalizedString("common.cancel", comment: ""), role: .cancel) {}
                Button(NSLocalizedString("entraide.action.confirm", comment: "")) {
                    Task {
                        if await viewModel.confirm(id: wish.id) {
                            showCelebration = true
                            try? await Task.sleep(for: .seconds(2))
                            showCelebration = false
                            dismiss()
                        }
                    }
                }
            } message: {
                Text(String(
                    format: NSLocalizedString("entraide.fulfill.confirmMessage", comment: ""),
                    wish.matchedWithDisplayName ?? ""
                ))
            }
            HStack(spacing: OffriiTheme.spacingSM) {
                OffriiButton(NSLocalizedString("entraide.action.messages", comment: ""), variant: .secondary) {
                    openMessages()
                }
                OffriiButton(NSLocalizedString("entraide.action.reject", comment: ""), variant: .danger) {
                    Task { _ = await viewModel.rejectOffer(id: wish.id) }
                }
            }
        }
    }

    @ViewBuilder
    private func donorMatchedActions(_ wish: WishDetail) -> some View {
        if wish.status == .matched && isMatchedByMe && !isMine {
            OffriiButton(NSLocalizedString("entraide.action.messages", comment: ""), variant: .primary) {
                openMessages()
            }
            OffriiButton(NSLocalizedString("entraide.action.withdraw", comment: ""), variant: .danger) {
                showWithdrawConfirm = true
            }
            .alert(
                NSLocalizedString("entraide.withdraw.confirmTitle", comment: ""),
                isPresented: $showWithdrawConfirm
            ) {
                Button(NSLocalizedString("common.cancel", comment: ""), role: .cancel) {}
                Button(NSLocalizedString("entraide.action.withdraw", comment: ""), role: .destructive) {
                    Task { _ = await viewModel.withdrawOffer(id: wish.id) }
                }
            } message: {
                Text(NSLocalizedString("entraide.withdraw.confirmMessage", comment: ""))
            }
        }
    }

    @ViewBuilder
    private func ownerOpenActions(_ wish: WishDetail) -> some View {
        if isMine && [.open, .review, .closed].contains(wish.status) {
            OffriiButton(NSLocalizedString("entraide.action.edit", comment: ""), variant: .secondary) {
                showEditSheet = true
            }
        }
        if isMine && wish.status == .open {
            OffriiButton(NSLocalizedString("entraide.action.close", comment: ""), variant: .ghost) {
                showCloseConfirm = true
            }
            .alert(
                NSLocalizedString("entraide.close.confirmTitle", comment: ""),
                isPresented: $showCloseConfirm
            ) {
                Button(NSLocalizedString("common.cancel", comment: ""), role: .cancel) {}
                Button(NSLocalizedString("entraide.action.close", comment: ""), role: .destructive) {
                    Task { _ = await viewModel.closeWish(id: wish.id) }
                }
            } message: {
                Text(NSLocalizedString("entraide.close.confirmMessage", comment: ""))
            }
        }
        if isMine && (wish.status == .review || wish.status == .closed) {
            OffriiButton(NSLocalizedString("entraide.action.reopen", comment: ""), variant: .secondary) {
                Task { _ = await viewModel.reopenWish(id: wish.id) }
            }
            if wish.status == .review {
                OffriiButton(NSLocalizedString("entraide.action.close", comment: ""), variant: .ghost) {
                    showCloseConfirm = true
                }
            }
        }
        // Delete — available for open, closed, pending, review, flagged, rejected
        if isMine && wish.status != .matched && wish.status != .fulfilled {
            OffriiButton(NSLocalizedString("entraide.action.delete", comment: ""), variant: .danger) {
                showDeleteConfirm = true
            }
            .alert(
                NSLocalizedString("entraide.delete.confirmTitle", comment: ""),
                isPresented: $showDeleteConfirm
            ) {
                Button(NSLocalizedString("common.cancel", comment: ""), role: .cancel) {}
                Button(NSLocalizedString("entraide.action.delete", comment: ""), role: .destructive) {
                    Task {
                        if await viewModel.deleteWish(id: wish.id) {
                            dismiss()
                        }
                    }
                }
            } message: {
                Text(NSLocalizedString("entraide.delete.confirmMessage", comment: ""))
            }
        }
    }

    @ViewBuilder
    private func reportAction(_ wish: WishDetail) -> some View {
        if !isMine && wish.status == .open && isUserEligible {
            if wish.hasReported ?? false {
                Label(NSLocalizedString("entraide.report.alreadyReported", comment: ""), systemImage: "checkmark.shield")
                    .font(OffriiTypography.footnote)
                    .foregroundColor(OffriiTheme.textMuted)
                    .frame(maxWidth: .infinity, alignment: .center)
                    .padding(.top, OffriiTheme.spacingSM)
            } else {
                OffriiButton(NSLocalizedString("entraide.report.title", comment: ""), variant: .ghost) {
                    openReport()
                }
                .padding(.top, OffriiTheme.spacingSM)
            }
        }
    }

    // MARK: - Messages / Report Navigation

    private func openMessages() {
        if let onOpenMessages {
            // Parent handles the sheet (EntraideView context)
            dismiss()
            DispatchQueue.main.asyncAfter(deadline: .now() + 0.3) { onOpenMessages() }
        } else {
            // Autonomous mode — open internally
            showMessagesSheet = true
        }
    }

    private func openReport() {
        if let onReport {
            dismiss()
            DispatchQueue.main.asyncAfter(deadline: .now() + 0.3) { onReport() }
        } else {
            showReportSheet = true
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
}
