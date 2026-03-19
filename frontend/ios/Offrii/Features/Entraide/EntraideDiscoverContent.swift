import SwiftUI

// MARK: - Discover Content

struct EntraideDiscoverContent: View {
    var viewModel: EntraideViewModel
    @Binding var selectedWishId: UUID?

    private let gridColumns = [
        GridItem(.flexible(), spacing: OffriiTheme.spacingSM),
        GridItem(.flexible(), spacing: OffriiTheme.spacingSM)
    ]

    var body: some View {
        ScrollView {
            LazyVStack(spacing: 0, pinnedViews: .sectionHeaders) {
                Section {
                    if viewModel.isLoading && viewModel.wishes.isEmpty {
                        skeletonGrid
                    } else if viewModel.wishes.isEmpty {
                        Spacer().frame(height: 60)
                        OffriiEmptyState(
                            icon: "heart.circle",
                            title: NSLocalizedString("entraide.empty", comment: ""),
                            subtitle: NSLocalizedString("entraide.emptySubtitle", comment: "")
                        )
                        Spacer()
                    } else {
                        wishGrid
                    }
                } header: {
                    categoryChips
                        .background(OffriiTheme.background)
                }
            }
        }
        .refreshable {
            await viewModel.loadWishes()
        }
    }

    // MARK: - Category Chips (exact same pattern as WishlistView)

    private var categoryChips: some View {
        ScrollView(.horizontal, showsIndicators: false) {
            HStack(spacing: OffriiTheme.spacingSM) {
                let allSelected = viewModel.selectedCategory == nil

                Button {
                    Task { await viewModel.selectCategory(nil) }
                } label: {
                    HStack(spacing: 4) {
                        Image(systemName: "sparkles")
                            .font(.system(size: 11))
                        Text(NSLocalizedString("entraide.category.all", comment: ""))
                            .font(.system(size: 13, weight: allSelected ? .semibold : .regular))
                    }
                    .foregroundColor(allSelected ? .white : OffriiTheme.textSecondary)
                    .padding(.horizontal, OffriiTheme.spacingMD)
                    .padding(.vertical, OffriiTheme.spacingSM)
                    .background(allSelected ? OffriiTheme.primary : .white)
                    .cornerRadius(OffriiTheme.cornerRadiusXL)
                    .overlay(
                        RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusXL)
                            .strokeBorder(allSelected ? .clear : OffriiTheme.border, lineWidth: 1)
                    )
                }
                .buttonStyle(.plain)

                ForEach(WishCategory.allCases) { category in
                    let isSelected = viewModel.selectedCategory == category
                    let color = entraideCategoryColor(category)

                    HStack(spacing: 4) {
                        Image(systemName: entraideCategoryIcon(category))
                            .font(.system(size: 11))
                        Text(category.label)
                            .font(.system(size: 13, weight: isSelected ? .semibold : .regular))
                    }
                    .foregroundColor(isSelected ? .white : OffriiTheme.textSecondary)
                    .padding(.horizontal, OffriiTheme.spacingMD)
                    .padding(.vertical, OffriiTheme.spacingSM)
                    .background(isSelected ? color : .white)
                    .cornerRadius(OffriiTheme.cornerRadiusXL)
                    .overlay(
                        RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusXL)
                            .strokeBorder(isSelected ? .clear : OffriiTheme.border, lineWidth: 1)
                    )
                    .onTapGesture {
                        Task { await viewModel.selectCategory(category) }
                    }
                    .animation(OffriiAnimation.snappy, value: isSelected)
                }
            }
            .padding(.horizontal, OffriiTheme.spacingBase)
            .padding(.vertical, OffriiTheme.spacingXS)
        }
    }

    // MARK: - Wish Grid

    private var wishGrid: some View {
        LazyVGrid(columns: gridColumns, spacing: OffriiTheme.spacingSM) {
            ForEach(viewModel.filteredWishes) { wish in
                EntraideWishCard(wish: wish) {
                    selectedWishId = wish.id
                }
                .onAppear {
                    Task { await viewModel.loadMoreIfNeeded(currentWish: wish) }
                }
            }
        }
        .padding(.horizontal, OffriiTheme.spacingBase)
        .padding(.vertical, OffriiTheme.spacingSM)
    }

    // MARK: - Skeleton

    private var skeletonGrid: some View {
        LazyVGrid(columns: gridColumns, spacing: OffriiTheme.spacingSM) {
            ForEach(0..<6, id: \.self) { _ in
                SkeletonGridCard()
            }
        }
        .padding(.horizontal, OffriiTheme.spacingBase)
        .padding(.vertical, OffriiTheme.spacingSM)
    }

    // MARK: - Category Helpers

    private func entraideCategoryColor(_ cat: WishCategory) -> Color {
        switch cat {
        case .education: return OffriiTheme.categoryEducationBg
        case .clothing:  return OffriiTheme.categoryClothingBg
        case .health:    return OffriiTheme.categoryHealthBg
        case .religion:  return OffriiTheme.categoryReligionBg
        case .home:      return OffriiTheme.categoryHomeBg
        case .children:  return OffriiTheme.categoryChildrenBg
        case .other:     return OffriiTheme.categoryOtherBg
        }
    }

    private func entraideCategoryIcon(_ cat: WishCategory) -> String {
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
