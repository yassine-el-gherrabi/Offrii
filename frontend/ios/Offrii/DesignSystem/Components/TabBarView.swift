import SwiftUI

// MARK: - Tab Item Enum

enum TabItem: Int, CaseIterable, Identifiable {
    case envies
    case cercles
    case entraide
    case profil

    var id: Int { rawValue }

    var label: LocalizedStringKey {
        switch self {
        case .envies:   return "Envies"
        case .cercles:  return "Cercles"
        case .entraide: return "Entraide"
        case .profil:   return "Profil"
        }
    }

    var iconName: String {
        switch self {
        case .envies:   return "heart.fill"
        case .cercles:  return "person.2.fill"
        case .entraide: return "hand.raised.fill"
        case .profil:   return "person.fill"
        }
    }
}

// MARK: - TabBarView

struct TabBarView: View {
    @Binding var selectedTab: TabItem
    @Namespace private var tabNamespace

    var body: some View {
        HStack {
            ForEach(TabItem.allCases) { tab in
                tabButton(for: tab)
            }
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
