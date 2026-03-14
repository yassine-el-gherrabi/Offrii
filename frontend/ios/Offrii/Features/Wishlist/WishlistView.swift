import SwiftUI

struct WishlistView: View {
    @Environment(AuthManager.self) private var authManager
    @Environment(OnboardingTipManager.self) private var tipManager
    @State private var viewModel = WishlistViewModel()
    @State private var showQuickAdd = false
    @State private var showSortMenu = false

    var body: some View {
        ZStack(alignment: .bottomTrailing) {
            OffriiTheme.background.ignoresSafeArea()

            VStack(spacing: 0) {
                // Header
                SectionHeader(
                    title: NSLocalizedString("wishlist.title", comment: ""),
                    variant: .envies
                ) {
                    NavigationLink(destination: ProfileView()) {
                        ProfileAvatarButton(
                            initials: ProfileAvatarButton.initials(from: authManager.currentUser?.displayName)
                        )
                    }
                }

                // Category chips
                ScrollView(.horizontal, showsIndicators: false) {
                    HStack(spacing: OffriiTheme.spacingSM) {
                        OffriiChip(
                            title: NSLocalizedString("wishlist.allCategories", comment: ""),
                            isSelected: viewModel.selectedCategoryId == nil
                        ) {
                            viewModel.selectCategory(nil)
                        }

                        ForEach(viewModel.categories, id: \.id) { category in
                            OffriiChip(
                                title: category.name,
                                isSelected: viewModel.selectedCategoryId == category.id
                            ) {
                                viewModel.selectCategory(category.id)
                            }
                        }
                    }
                    .padding(.horizontal, OffriiTheme.spacingLG)
                    .padding(.vertical, OffriiTheme.spacingSM)
                }

                // Segmented + sort
                HStack {
                    Picker("", selection: Binding(
                        get: { viewModel.filteredSegmentIndex },
                        set: { viewModel.changeSegment($0) }
                    )) {
                        Text(NSLocalizedString("wishlist.active", comment: "")).tag(0)
                        Text(NSLocalizedString("wishlist.purchased", comment: "")).tag(1)
                    }
                    .pickerStyle(.segmented)

                    Menu {
                        Button {
                            viewModel.changeSort("created_at")
                        } label: {
                            Label(
                                NSLocalizedString("wishlist.sort.date", comment: ""),
                                systemImage: viewModel.sortField == "created_at" ? "checkmark" : ""
                            )
                        }
                        Button {
                            viewModel.changeSort("priority")
                        } label: {
                            Label(
                                NSLocalizedString("wishlist.sort.priority", comment: ""),
                                systemImage: viewModel.sortField == "priority" ? "checkmark" : ""
                            )
                        }
                        Button {
                            viewModel.changeSort("name")
                        } label: {
                            Label(
                                NSLocalizedString("wishlist.sort.name", comment: ""),
                                systemImage: viewModel.sortField == "name" ? "checkmark" : ""
                            )
                        }
                    } label: {
                        Image(systemName: "arrow.up.arrow.down")
                            .foregroundColor(OffriiTheme.primary)
                            .padding(OffriiTheme.spacingSM)
                    }
                }
                .padding(.horizontal, OffriiTheme.spacingLG)
                .padding(.vertical, OffriiTheme.spacingSM)

                // List
                if viewModel.isLoading && viewModel.items.isEmpty {
                    Spacer()
                    SkeletonList()
                    Spacer()
                } else if viewModel.items.isEmpty {
                    Spacer()
                    OffriiEmptyState(
                        icon: "gift",
                        title: NSLocalizedString("wishlist.empty", comment: ""),
                        subtitle: NSLocalizedString("wishlist.emptySubtitle", comment: ""),
                        ctaTitle: NSLocalizedString("wishlist.quickAdd.button", comment: ""),
                        ctaAction: { showQuickAdd = true }
                    )
                    Spacer()
                } else {
                    List {
                        ForEach(Array(viewModel.items.enumerated()), id: \.element.id) { index, item in
                            NavigationLink(value: item.id) {
                                ItemRow(
                                    item: item,
                                    categoryName: viewModel.categoryName(for: item.categoryId)
                                )
                            }
                            .listRowBackground(OffriiTheme.card)
                            .listRowSeparatorTint(OffriiTheme.border)
                            .overlay(alignment: .top) {
                                if index == 0, tipManager.activeTip == .wishlistSwipe {
                                    OffriiTooltip(
                                        message: OnboardingTipManager.message(for: .wishlistSwipe),
                                        arrow: .top
                                    ) {
                                        tipManager.dismiss(.wishlistSwipe)
                                    }
                                    .offset(y: -60)
                                }
                            }
                            .swipeActions(edge: .trailing, allowsFullSwipe: false) {
                                Button(role: .destructive) {
                                    Task { await viewModel.deleteItem(item) }
                                } label: {
                                    Label(
                                        NSLocalizedString("common.delete", comment: ""),
                                        systemImage: "trash"
                                    )
                                }
                            }
                            .swipeActions(edge: .leading, allowsFullSwipe: true) {
                                if item.isActive {
                                    Button {
                                        Task { await viewModel.markPurchased(item) }
                                    } label: {
                                        Label(
                                            NSLocalizedString("wishlist.markReceived", comment: ""),
                                            systemImage: "checkmark.circle"
                                        )
                                    }
                                    .tint(OffriiTheme.success)
                                }
                            }
                            .onAppear {
                                Task { await viewModel.loadMoreIfNeeded(currentItem: item) }
                            }
                        }

                        if viewModel.isLoadingMore {
                            HStack {
                                Spacer()
                                SkeletonRow()
                                Spacer()
                            }
                            .listRowBackground(Color.clear)
                        }
                    }
                    .listStyle(.plain)
                    .scrollContentBackground(.hidden)
                    .navigationDestination(for: UUID.self) { itemId in
                        ItemDetailView(itemId: itemId)
                            .environment(authManager)
                    }
                }
            }

            // FAB
            VStack {
                Spacer()
                HStack {
                    Spacer()
                    OffriiFloatingActionButton(icon: "plus") {
                        showQuickAdd = true
                    }
                    .overlay(alignment: .top) {
                        if tipManager.activeTip == .wishlistFirstAdd {
                            OffriiTooltip(
                                message: OnboardingTipManager.message(for: .wishlistFirstAdd),
                                arrow: .bottom
                            ) {
                                tipManager.dismiss(.wishlistFirstAdd)
                            }
                            .offset(y: -80)
                        }
                    }
                }
                .padding(.trailing, OffriiTheme.spacingLG)
                .padding(.bottom, OffriiTheme.spacingLG)
            }
        }
        .refreshable {
            await viewModel.loadItems()
        }
        .sheet(isPresented: $showQuickAdd) {
            QuickAddSheet { name in
                await viewModel.quickAdd(name: name)
            }
        }
        .task {
            await viewModel.loadCategories()
            await viewModel.loadItems()
            if viewModel.items.isEmpty {
                tipManager.showIfNeeded(.wishlistFirstAdd)
            } else {
                tipManager.showIfNeeded(.wishlistSwipe)
            }
        }
    }
}
