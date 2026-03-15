import SwiftUI

// MARK: - ItemDetailSheet

struct ItemDetailSheet: View {
    let itemId: UUID
    @Environment(AuthManager.self) private var authManager
    @Environment(\.dismiss) private var dismiss
    @State private var viewModel = ItemDetailViewModel()
    @State private var showEdit = false
    @State private var showDeleteAlert = false
    @State private var showShareToCircle = false

    var body: some View {
        NavigationStack {
            ZStack {
                OffriiTheme.background.ignoresSafeArea()

                if viewModel.isLoading {
                    VStack(spacing: OffriiTheme.spacingBase) {
                        SkeletonRow(height: 140)
                        SkeletonRow()
                        SkeletonRow()
                    }
                    .padding(.horizontal, OffriiTheme.spacingLG)
                } else if let item = viewModel.item {
                    ScrollView {
                        VStack(spacing: 0) {
                            // Image hero
                            heroImage(item)

                            // Title + price
                            VStack(alignment: .leading, spacing: OffriiTheme.spacingSM) {
                                Text(item.name)
                                    .font(OffriiTypography.title3)
                                    .foregroundColor(OffriiTheme.text)

                                if let price = item.estimatedPrice {
                                    Text(formatPrice(price))
                                        .font(OffriiTypography.headline)
                                        .foregroundColor(OffriiTheme.primary)
                                }
                            }
                            .frame(maxWidth: .infinity, alignment: .leading)
                            .padding(.horizontal, OffriiTheme.spacingLG)
                            .padding(.top, OffriiTheme.spacingBase)

                            // Chips
                            HStack(spacing: OffriiTheme.spacingSM) {
                                if let catName = viewModel.categoryName {
                                    chipLabel(catName, bgColor: OffriiTheme.primary.opacity(0.1), fgColor: OffriiTheme.primary)
                                }
                                priorityChip(item.priority)
                            }
                            .frame(maxWidth: .infinity, alignment: .leading)
                            .padding(.horizontal, OffriiTheme.spacingLG)
                            .padding(.top, OffriiTheme.spacingSM)

                            // Claimed banner
                            if item.isClaimed {
                                if item.isWebClaim, let name = item.claimedName {
                                    // Web claim — corail, informative for the owner
                                    VStack(spacing: OffriiTheme.spacingSM) {
                                        HStack {
                                            Image(systemName: "link")
                                            Text(String(format: NSLocalizedString("wishlist.claimed.web", comment: ""), name))
                                                .font(OffriiTypography.subheadline)
                                                .fontWeight(.semibold)
                                        }
                                        .foregroundColor(OffriiTheme.primary)

                                        Button {
                                            Task {
                                                await viewModel.ownerUnclaimWeb()
                                            }
                                        } label: {
                                            HStack(spacing: OffriiTheme.spacingXS) {
                                                Image(systemName: "xmark.circle")
                                                Text(NSLocalizedString("wishlist.unclaim.web", comment: ""))
                                            }
                                            .font(OffriiTypography.caption)
                                            .fontWeight(.medium)
                                            .foregroundColor(OffriiTheme.textSecondary)
                                        }
                                    }
                                    .frame(maxWidth: .infinity)
                                    .padding(.vertical, OffriiTheme.spacingSM)
                                    .background(OffriiTheme.primary.opacity(0.08))
                                    .cornerRadius(OffriiTheme.cornerRadiusSM)
                                    .padding(.horizontal, OffriiTheme.spacingLG)
                                    .padding(.top, OffriiTheme.spacingBase)
                                } else {
                                    // App claim — corail, anti-spoiler
                                    HStack {
                                        Image(systemName: "hand.thumbsup.fill")
                                        Text(NSLocalizedString("wishlist.claimed", comment: ""))
                                            .font(OffriiTypography.subheadline)
                                            .fontWeight(.semibold)
                                    }
                                    .foregroundColor(OffriiTheme.primary)
                                    .frame(maxWidth: .infinity)
                                    .padding(.vertical, OffriiTheme.spacingSM)
                                    .background(OffriiTheme.primary.opacity(0.08))
                                    .cornerRadius(OffriiTheme.cornerRadiusSM)
                                    .padding(.horizontal, OffriiTheme.spacingLG)
                                    .padding(.top, OffriiTheme.spacingBase)
                                }
                            }

                            // Private banner
                            if item.isPrivate {
                                HStack {
                                    Image(systemName: "lock.fill")
                                    Text(NSLocalizedString("wishlist.privateHint", comment: ""))
                                        .font(OffriiTypography.subheadline)
                                        .fontWeight(.medium)
                                }
                                .foregroundColor(OffriiTheme.textSecondary)
                                .frame(maxWidth: .infinity)
                                .padding(.vertical, OffriiTheme.spacingSM)
                                .background(OffriiTheme.surface)
                                .cornerRadius(OffriiTheme.cornerRadiusSM)
                                .padding(.horizontal, OffriiTheme.spacingLG)
                                .padding(.top, OffriiTheme.spacingSM)
                            }

                            // Description
                            if let desc = item.description, !desc.isEmpty {
                                VStack(alignment: .leading, spacing: OffriiTheme.spacingXS) {
                                    Text(NSLocalizedString("item.description", comment: ""))
                                        .font(OffriiTypography.subheadline)
                                        .foregroundColor(OffriiTheme.textMuted)
                                    Text(desc)
                                        .font(OffriiTypography.body)
                                        .foregroundColor(OffriiTheme.text)
                                }
                                .frame(maxWidth: .infinity, alignment: .leading)
                                .padding(.horizontal, OffriiTheme.spacingLG)
                                .padding(.top, OffriiTheme.spacingBase)
                            }

                            // Links section
                            linksSection(item)

                            // Shared with section
                            sharedWithSection(item)

                            // Date
                            HStack(spacing: OffriiTheme.spacingXS) {
                                Image(systemName: "calendar")
                                    .font(.system(size: 12))
                                    .foregroundColor(OffriiTheme.textMuted)
                                Text(item.createdAt, style: .date)
                                    .font(OffriiTypography.caption)
                                    .foregroundColor(OffriiTheme.textMuted)
                            }
                            .frame(maxWidth: .infinity, alignment: .leading)
                            .padding(.horizontal, OffriiTheme.spacingLG)
                            .padding(.top, OffriiTheme.spacingBase)

                            // Actions
                            VStack(spacing: OffriiTheme.spacingSM) {
                                if item.isActive {
                                    OffriiButton(
                                        NSLocalizedString("wishlist.markReceived", comment: ""),
                                        variant: .primary,
                                        isLoading: viewModel.isUpdating
                                    ) {
                                        Task {
                                            if await viewModel.markPurchased() {
                                                dismiss()
                                            }
                                        }
                                    }
                                }

                                HStack(spacing: OffriiTheme.spacingSM) {
                                    Button {
                                        showEdit = true
                                    } label: {
                                        HStack(spacing: OffriiTheme.spacingXS) {
                                            Image(systemName: "pencil")
                                            Text(NSLocalizedString("wishlist.edit", comment: ""))
                                        }
                                        .font(OffriiTypography.subheadline)
                                        .fontWeight(.medium)
                                        .foregroundColor(OffriiTheme.primary)
                                        .frame(maxWidth: .infinity)
                                        .padding(.vertical, OffriiTheme.spacingSM)
                                        .background(OffriiTheme.primary.opacity(0.1))
                                        .cornerRadius(OffriiTheme.cornerRadiusSM)
                                    }

                                    Button {
                                        showDeleteAlert = true
                                    } label: {
                                        HStack(spacing: OffriiTheme.spacingXS) {
                                            Image(systemName: "trash")
                                            Text(NSLocalizedString("common.delete", comment: ""))
                                        }
                                        .font(OffriiTypography.subheadline)
                                        .fontWeight(.medium)
                                        .foregroundColor(OffriiTheme.danger)
                                        .frame(maxWidth: .infinity)
                                        .padding(.vertical, OffriiTheme.spacingSM)
                                        .background(OffriiTheme.danger.opacity(0.1))
                                        .cornerRadius(OffriiTheme.cornerRadiusSM)
                                    }
                                }
                            }
                            .padding(.horizontal, OffriiTheme.spacingLG)
                            .padding(.top, OffriiTheme.spacingLG)
                            .padding(.bottom, OffriiTheme.spacingXL)
                        }
                    }
                }
            }
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button {
                        dismiss()
                    } label: {
                        Image(systemName: "xmark.circle.fill")
                            .font(.system(size: 24))
                            .symbolRenderingMode(.hierarchical)
                            .foregroundColor(OffriiTheme.textMuted)
                    }
                }
            }
            .alert(
                NSLocalizedString("wishlist.delete.title", comment: ""),
                isPresented: $showDeleteAlert
            ) {
                Button(NSLocalizedString("common.cancel", comment: ""), role: .cancel) {}
                Button(NSLocalizedString("wishlist.delete.confirm", comment: ""), role: .destructive) {
                    Task {
                        if await viewModel.deleteItem() {
                            dismiss()
                        }
                    }
                }
            } message: {
                Text(NSLocalizedString("wishlist.delete.message", comment: ""))
            }
            .sheet(isPresented: $showEdit) {
                if let item = viewModel.item {
                    NavigationStack {
                        ItemEditView(item: item) { updatedItem in
                            viewModel.item = updatedItem
                        }
                    }
                }
            }
            .sheet(isPresented: $showShareToCircle) {
                if let item = viewModel.item {
                    ShareToCircleSheet(
                        itemId: item.id,
                        alreadySharedCircleIds: Set(item.sharedCircles.map(\.id))
                    )
                    .presentationDetents([.medium])
                    .onDisappear {
                        Task { await viewModel.loadItem(id: itemId) }
                    }
                }
            }
            .task {
                await viewModel.loadItem(id: itemId)
            }
        }
    }

    // MARK: - Hero Image

    @ViewBuilder
    private func heroImage(_ item: Item) -> some View {
        ZStack {
            Group {
                if let url = item.displayImageUrl {
                    AsyncImage(url: url) { phase in
                        switch phase {
                        case .success(let image):
                            image
                                .resizable()
                                .aspectRatio(contentMode: .fill)
                                .frame(height: 200)
                                .clipped()
                        default:
                            categoryGradientView
                        }
                    }
                } else {
                    categoryGradientView
                }
            }

            if item.isClaimed {
                Color.black.opacity(0.35)
                Text(NSLocalizedString("wishlist.reserved", comment: ""))
                    .font(.system(size: 16, weight: .bold))
                    .tracking(3)
                    .textCase(.uppercase)
                    .foregroundColor(.white)
            }
        }
        .frame(maxWidth: .infinity)
        .frame(height: 200)
        .clipped()
    }

    private var categoryGradientView: some View {
        // Use a neutral gradient since we don't have the category here
        // (would need to pass it or load it)
        LinearGradient(
            colors: [OffriiTheme.primary.opacity(0.3), OffriiTheme.accent.opacity(0.2)],
            startPoint: .topLeading,
            endPoint: .bottomTrailing
        )
        .frame(height: 200)
        .overlay(
            Image(systemName: "gift.fill")
                .font(.system(size: 48, weight: .light))
                .foregroundColor(.white.opacity(0.6))
        )
    }

    // MARK: - Shared With Section

    @ViewBuilder
    private func sharedWithSection(_ item: Item) -> some View {
        VStack(alignment: .leading, spacing: OffriiTheme.spacingSM) {
            Text(NSLocalizedString("item.sharedWith", comment: ""))
                .font(OffriiTypography.subheadline)
                .foregroundColor(OffriiTheme.textMuted)

            ForEach(item.sharedCircles) { circle in
                HStack(spacing: OffriiTheme.spacingSM) {
                    ZStack {
                        Text(circle.initial)
                            .font(.system(size: 12, weight: .bold))
                            .foregroundColor(.white)
                            .frame(width: 28, height: 28)
                            .background(circle.isDirect == true ? OffriiTheme.textSecondary : OffriiTheme.primary)
                            .clipShape(Circle())
                    }
                    .overlay(alignment: .bottomTrailing) {
                        Image(systemName: circle.isDirect == true ? "person.fill" : "person.2.fill")
                            .font(.system(size: 7, weight: .bold))
                            .foregroundColor(.white)
                            .padding(2)
                            .background(circle.isDirect == true ? OffriiTheme.textSecondary : OffriiTheme.primary)
                            .clipShape(Circle())
                            .overlay(Circle().strokeBorder(.white, lineWidth: 1))
                            .offset(x: 3, y: 3)
                    }

                    Text(circle.name)
                        .font(.system(size: 14, weight: .medium))
                        .foregroundColor(OffriiTheme.text)

                    Spacer()

                    Button {
                        Task {
                            await viewModel.unshareFromCircle(circleId: circle.id)
                        }
                    } label: {
                        Image(systemName: "xmark")
                            .font(.system(size: 11, weight: .semibold))
                            .foregroundColor(OffriiTheme.textMuted)
                    }
                }
                .padding(OffriiTheme.spacingSM)
                .background(OffriiTheme.surface)
                .cornerRadius(OffriiTheme.cornerRadiusMD)
            }

            // Add row — iOS standard pattern
            Button {
                showShareToCircle = true
            } label: {
                HStack(spacing: OffriiTheme.spacingSM) {
                    Image(systemName: "plus")
                        .font(.system(size: 13, weight: .semibold))
                        .foregroundColor(.white)
                        .frame(width: 28, height: 28)
                        .background(OffriiTheme.primary)
                        .clipShape(Circle())

                    Text(NSLocalizedString("share.addPeople", comment: ""))
                        .font(.system(size: 14, weight: .medium))
                        .foregroundColor(OffriiTheme.primary)
                }
                .padding(OffriiTheme.spacingSM)
                .frame(maxWidth: .infinity, alignment: .leading)
                .background(OffriiTheme.surface)
                .cornerRadius(OffriiTheme.cornerRadiusMD)
            }
            .buttonStyle(.plain)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(.horizontal, OffriiTheme.spacingLG)
        .padding(.top, OffriiTheme.spacingBase)
    }

    // MARK: - Links Section

    @ViewBuilder
    private func linksSection(_ item: Item) -> some View {
        let links = item.links?.filter({ !$0.isEmpty }) ?? []
        if !links.isEmpty {
            VStack(alignment: .leading, spacing: OffriiTheme.spacingSM) {
                Text(NSLocalizedString("item.links", comment: ""))
                    .font(OffriiTypography.subheadline)
                    .foregroundColor(OffriiTheme.textMuted)

                ForEach(links, id: \.self) { link in
                    if let url = URL(string: link) {
                        Link(destination: url) {
                            HStack(spacing: OffriiTheme.spacingSM) {
                                Image(systemName: "globe")
                                    .font(.system(size: 14))
                                    .foregroundColor(OffriiTheme.primary)
                                    .frame(width: 28, height: 28)
                                    .background(OffriiTheme.primary.opacity(0.1))
                                    .cornerRadius(OffriiTheme.cornerRadiusSM)

                                VStack(alignment: .leading, spacing: 1) {
                                    Text(url.host ?? link)
                                        .font(.system(size: 13, weight: .medium))
                                        .foregroundColor(OffriiTheme.text)
                                        .lineLimit(1)

                                    if let ogTitle = item.ogTitle, !ogTitle.isEmpty {
                                        Text(ogTitle)
                                            .font(.system(size: 11))
                                            .foregroundColor(OffriiTheme.textMuted)
                                            .lineLimit(1)
                                    }
                                }

                                Spacer()

                                Image(systemName: "arrow.up.right")
                                    .font(.system(size: 11, weight: .semibold))
                                    .foregroundColor(OffriiTheme.textMuted)
                            }
                            .padding(OffriiTheme.spacingSM)
                            .background(OffriiTheme.surface)
                            .cornerRadius(OffriiTheme.cornerRadiusMD)
                        }
                    }
                }
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(.horizontal, OffriiTheme.spacingLG)
            .padding(.top, OffriiTheme.spacingBase)
        }
    }

    // MARK: - Helpers

    private func chipLabel(_ text: String, bgColor: Color, fgColor: Color) -> some View {
        Text(text)
            .font(OffriiTypography.caption)
            .fontWeight(.medium)
            .foregroundColor(fgColor)
            .padding(.horizontal, 10)
            .padding(.vertical, 4)
            .background(bgColor)
            .cornerRadius(OffriiTheme.cornerRadiusSM)
    }

    @ViewBuilder
    private func priorityChip(_ priority: Int) -> some View {
        HStack(spacing: 3) {
            ForEach(0..<priority, id: \.self) { _ in
                Circle()
                    .fill(priority == 3 ? OffriiTheme.danger : (priority == 2 ? OffriiTheme.accent : OffriiTheme.textMuted))
                    .frame(width: 6, height: 6)
            }
            Text(Item.priorityLabelStatic(priority))
                .font(OffriiTypography.caption)
                .fontWeight(.medium)
                .foregroundColor(priority == 3 ? OffriiTheme.danger : (priority == 2 ? OffriiTheme.accent : OffriiTheme.textMuted))
        }
        .padding(.horizontal, 10)
        .padding(.vertical, 4)
        .background((priority == 3 ? OffriiTheme.danger : (priority == 2 ? OffriiTheme.accent : OffriiTheme.textMuted)).opacity(0.1))
        .cornerRadius(OffriiTheme.cornerRadiusSM)
    }

    private func formatPrice(_ price: Decimal) -> String {
        let formatter = NumberFormatter()
        formatter.numberStyle = .currency
        formatter.currencyCode = "EUR"
        return formatter.string(from: price as NSDecimalNumber) ?? "\(price) \u{20AC}"
    }
}
