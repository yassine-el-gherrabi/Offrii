import SwiftUI

// MARK: - Tab Item Enum

enum TabItem: Int, CaseIterable, Identifiable {
    case home = 0
    case envies = 1
    case create = 2
    case cercles = 3
    case entraide = 4

    var id: Int { rawValue }

    var label: LocalizedStringKey {
        switch self {
        case .home:     return "tab.home"
        case .envies:   return "tab.envies"
        case .create:   return "tab.create"
        case .cercles:  return "tab.cercles"
        case .entraide: return "tab.entraide"
        }
    }

    var iconName: String {
        switch self {
        case .home:     return "house.fill"
        case .envies:   return "heart.fill"
        case .create:   return "plus"
        case .cercles:  return "person.2.fill"
        case .entraide: return "hand.raised.fill"
        }
    }

    /// The 4 real navigable tabs (excluding the FAB)
    static var navigableTabs: [TabItem] {
        [.home, .envies, .cercles, .entraide]
    }
}

// MARK: - TabBarView

struct TabBarView: View {
    @Binding var selectedTab: TabItem
    var onCreateTap: () -> Void
    @Namespace private var tabNamespace

    var body: some View {
        HStack {
            tabButton(for: .home)
            tabButton(for: .envies)

            // FAB central — flush accent pill
            fabButton

            tabButton(for: .cercles)
            tabButton(for: .entraide)
        }
        .padding(.top, OffriiTheme.spacingSM)
        .padding(.bottom, OffriiTheme.spacingXS)
        .background(
            OffriiTheme.card
                .shadow(
                    color: Color.black.opacity(0.08),
                    radius: 8,
                    x: 0,
                    y: -2
                )
                .ignoresSafeArea(edges: .bottom)
        )
    }

    // MARK: - FAB Central Button (flush pill, vertically centered, no label)

    private var fabButton: some View {
        Button {
            OffriiHaptics.selection()
            onCreateTap()
        } label: {
            // Hidden VStack mirrors tabButton structure to get equal height
            VStack(spacing: 4) {
                Image(systemName: "plus")
                    .font(.system(size: 22))
                Text("tab.create")
                    .font(OffriiTypography.caption2)
                Circle()
                    .frame(width: 4, height: 4)
            }
            .hidden()
            .frame(maxWidth: .infinity)
            .overlay {
                ZStack {
                    RoundedRectangle(cornerRadius: 12)
                        .fill(OffriiTheme.primary)
                        .frame(width: 56, height: 34)

                    Image(systemName: "plus")
                        .font(.system(size: 20, weight: .bold))
                        .foregroundColor(.white)
                }
                // Nudge up slightly to discount the dot from visual center
                .offset(y: -4)
            }
        }
        .buttonStyle(.plain)
        .accessibilityLabel(Text("tab.create"))
    }

    // MARK: - Tab Button

    private func tabButton(for tab: TabItem) -> some View {
        Button {
            OffriiHaptics.selection()
            withAnimation(OffriiAnimation.snappy) {
                selectedTab = tab
            }
        } label: {
            VStack(spacing: 4) {
                Image(systemName: tab.iconName)
                    .font(.system(size: 22))
                    .foregroundColor(tabColor(for: tab))

                Text(tab.label)
                    .font(OffriiTypography.caption2)
                    .foregroundColor(tabColor(for: tab))

                // Dot indicator
                Circle()
                    .fill(selectedTab == tab ? OffriiTheme.primary : Color.clear)
                    .frame(width: 4, height: 4)
                    .matchedGeometryEffect(
                        id: selectedTab == tab ? "tabDot" : "tabDot_\(tab.rawValue)",
                        in: tabNamespace
                    )
            }
            .frame(maxWidth: .infinity)
        }
        .buttonStyle(.plain)
        .accessibilityLabel(tab.label)
    }

    // MARK: - Tab Color

    private func tabColor(for tab: TabItem) -> Color {
        selectedTab == tab ? OffriiTheme.primary : OffriiTheme.textMuted
    }
}
