import SwiftUI

struct ItemDetailView: View {
    let itemId: UUID
    @Environment(AuthManager.self) private var authManager
    @Environment(\.dismiss) private var dismiss
    @State private var viewModel = ItemDetailViewModel()
    @State private var showEdit = false
    @State private var showDeleteAlert = false
    @State private var showShareSheet = false

    var body: some View {
        ZStack {
            OffriiTheme.background.ignoresSafeArea()

            if viewModel.isLoading {
                VStack(spacing: OffriiTheme.spacingBase) {
                    SkeletonRow(height: 120)
                    SkeletonRow()
                    SkeletonRow()
                }
            } else if let item = viewModel.item {
                ScrollView {
                    VStack(spacing: 0) {
                        // Header
                        SectionHeader(
                            title: item.name,
                            variant: .detail
                        ) {
                            Image(systemName: "gift.fill")
                                .font(.system(size: 28))
                                .foregroundColor(.white.opacity(0.7))
                        }

                        // Chips below header
                        HStack(spacing: OffriiTheme.spacingSM) {
                            if let catName = viewModel.categoryName {
                                chipLabel(catName, bgColor: OffriiTheme.primary.opacity(0.1), fgColor: OffriiTheme.primary)
                            }
                            chipLabel(item.priorityLabel, bgColor: priorityColor(item.priority).opacity(0.1), fgColor: priorityColor(item.priority))
                        }
                        .padding(.horizontal, OffriiTheme.spacingLG)
                        .padding(.top, OffriiTheme.spacingSM)

                        // Claimed banner
                        if item.isClaimed {
                            HStack {
                                Image(systemName: "hand.thumbsup.fill")
                                Text(NSLocalizedString("wishlist.claimed", comment: ""))
                                    .font(OffriiTypography.subheadline)
                                    .fontWeight(.semibold)
                            }
                            .foregroundColor(OffriiTheme.accent)
                            .frame(maxWidth: .infinity)
                            .padding(.vertical, OffriiTheme.spacingSM)
                            .background(OffriiTheme.accent.opacity(0.1))
                        }

                        // Content card
                        OffriiCard {
                            VStack(alignment: .leading, spacing: OffriiTheme.spacingBase) {
                                if let price = item.estimatedPrice {
                                    detailRow(
                                        icon: "eurosign.circle",
                                        title: NSLocalizedString("item.estimatedPrice", comment: ""),
                                        value: formatPrice(price)
                                    )
                                }

                                if let url = item.url, !url.isEmpty {
                                    detailRow(
                                        icon: "link",
                                        title: NSLocalizedString("item.url", comment: ""),
                                        value: url,
                                        isLink: true
                                    )
                                }

                                if let desc = item.description, !desc.isEmpty {
                                    VStack(alignment: .leading, spacing: OffriiTheme.spacingXS) {
                                        Text(NSLocalizedString("item.description", comment: ""))
                                            .font(OffriiTypography.subheadline)
                                            .foregroundColor(OffriiTheme.textMuted)
                                        Text(desc)
                                            .font(OffriiTypography.body)
                                            .foregroundColor(OffriiTheme.text)
                                    }
                                }
                            }
                        }
                        .padding(.horizontal, OffriiTheme.spacingLG)
                        .padding(.top, OffriiTheme.spacingBase)

                        // Actions
                        VStack(spacing: OffriiTheme.spacingSM) {
                            if item.isActive {
                                OffriiButton(
                                    NSLocalizedString("wishlist.markReceived", comment: ""),
                                    variant: .secondary,
                                    isLoading: viewModel.isUpdating
                                ) {
                                    Task {
                                        if await viewModel.markPurchased() {
                                            dismiss()
                                        }
                                    }
                                }
                            }

                            OffriiButton(
                                NSLocalizedString("common.delete", comment: ""),
                                variant: .danger
                            ) {
                                showDeleteAlert = true
                            }
                        }
                        .padding(.horizontal, OffriiTheme.spacingLG)
                        .padding(.top, OffriiTheme.spacingBase)
                        .padding(.bottom, OffriiTheme.spacingXL)
                    }
                }
            }
        }
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            ToolbarItem(placement: .primaryAction) {
                Button(NSLocalizedString("wishlist.edit", comment: "")) {
                    showEdit = true
                }
                .foregroundColor(OffriiTheme.primary)
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
        .task {
            await viewModel.loadItem(id: itemId)
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

    private func detailRow(icon: String, title: String, value: String, isLink: Bool = false) -> some View {
        HStack(alignment: .top, spacing: OffriiTheme.spacingSM) {
            Image(systemName: icon)
                .foregroundColor(OffriiTheme.primary)
                .frame(width: 20)

            VStack(alignment: .leading, spacing: 2) {
                Text(title)
                    .font(OffriiTypography.caption)
                    .foregroundColor(OffriiTheme.textMuted)

                if isLink, let url = URL(string: value) {
                    Link(value, destination: url)
                        .font(OffriiTypography.body)
                        .foregroundColor(OffriiTheme.primary)
                        .lineLimit(1)
                } else {
                    Text(value)
                        .font(OffriiTypography.body)
                        .foregroundColor(OffriiTheme.text)
                }
            }

            Spacer()
        }
    }

    private func priorityColor(_ priority: Int) -> Color {
        switch priority {
        case 1: return OffriiTheme.primary.opacity(0.4)
        case 3: return OffriiTheme.primary
        default: return OffriiTheme.primary.opacity(0.7)
        }
    }

    private func formatPrice(_ price: Decimal) -> String {
        let formatter = NumberFormatter()
        formatter.numberStyle = .currency
        formatter.currencyCode = "EUR"
        return formatter.string(from: price as NSDecimalNumber) ?? "\(price) \u{20AC}"
    }
}
