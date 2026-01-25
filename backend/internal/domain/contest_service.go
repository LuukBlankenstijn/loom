package domain

import "context"

type TeamService struct {
	teamRepo    TeamRepository
	contestRepo ContestRepository
}

func NewTeamService(tr TeamRepository, cr ContestRepository) *TeamService {
	return &TeamService{teamRepo: tr, contestRepo: cr}
}

func (s *TeamService) GetTeamsForActiveContest(ctx context.Context) ([]Team, error) {
	contest, err := s.contestRepo.GetNextContest(ctx)
	if err != nil {
		return []Team{}, err
	}
	// if contest is not found, return empty list
	if contest == nil {
		return []Team{}, nil
	}

	return s.teamRepo.GetAll(ctx, contest.Id)
}
