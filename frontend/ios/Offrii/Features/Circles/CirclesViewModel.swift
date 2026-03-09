import Foundation

@Observable
@MainActor
final class CirclesViewModel {
    var circles: [OffriiCircle] = []
    var pendingRequestsCount: Int = 0
    var isLoading = false
    var error: String?

    func loadCircles() async {
        isLoading = true
        error = nil
        do {
            circles = try await CircleService.shared.listCircles()
        } catch {
            self.error = error.localizedDescription
        }
        isLoading = false
    }

    func loadPendingCount() async {
        do {
            let requests = try await FriendService.shared.listPendingRequests()
            pendingRequestsCount = requests.count
        } catch {
            // Silently fail for badge count
        }
    }

    func deleteCircle(_ circle: OffriiCircle) async {
        do {
            try await CircleService.shared.deleteCircle(id: circle.id)
            circles.removeAll { $0.id == circle.id }
        } catch {
            self.error = error.localizedDescription
        }
    }
}
